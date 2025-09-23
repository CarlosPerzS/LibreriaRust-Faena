#[allow(dead_code)]
mod login;
mod modelo;
pub mod p2pcon;
#[allow(dead_code)]
mod registrar;
use jni::JNIEnv;
use jni::JavaVM;
use jni::objects::{GlobalRef, JClass, JObject, JString, JValue};
use jni::sys::{JNI_VERSION_1_6, jint};
use login::verificar_credenciales;
use once_cell::sync::OnceCell;
#[allow(unused_imports)]
use registrar::{registrar_usuario, validar_usuario};
use std::ffi::c_void;
#[allow(unused_imports)]
use std::sync::Arc;
use tokio::runtime::Runtime;
use tokio::time::{Duration, timeout};

use crate::modelo::UsuarioGuardado;
use crate::p2pcon::P2PMessage;

//variables publicas para la conexion entre la app y libreria
pub static TOKIO_RUNTIME: OnceCell<Runtime> = OnceCell::new(); //gestor de hilos principal
pub static JVM: OnceCell<JavaVM> = OnceCell::new(); //jvm de parte de android, las almacenamos en OnceCell para evitar cambios repentinos
static LOGGER_INIT: std::sync::Once = std::sync::Once::new();

//funcion para inicializar la variable publica de JVM para poder tener tareas asincronas como las peticiones reqwest e inicializacion en general
#[unsafe(no_mangle)]
pub extern "system" fn JNI_OnLoad(vm: JavaVM, _: *mut c_void) -> jint {
    //recibimos la JVM de java android
    init_logger(); // 
    TOKIO_RUNTIME.get_or_init(|| Runtime::new().unwrap()); //incializacion de runtime de tokio
    if JVM.set(vm).is_err() {
        //inicializacion de la jvm (variable global de la libreria)
        return -1; // JNI_ERR
    }
    JNI_VERSION_1_6
}

fn init_logger() {
    #[cfg(target_os = "android")]
    {
        LOGGER_INIT.call_once(|| {
            android_logger::init_once(
                android_logger::Config::default()
                    .with_max_level(log::LevelFilter::Debug)
                    .with_tag("Rust"),
            );
        });
    }
}
//se necesita que el nombre sea Java_paquete_clase_nombre de la funcion
//funcion para iniciar sesion en la bd desde rust
#[unsafe(no_mangle)]
pub extern "C" fn Java_com_example_faena_login_login(
    mut env: JNIEnv,
    this: JObject,
    correo: JString,
    pswd: JString,
) {
    //en env y class/this son argumentos dados por JNI cuando se invoca a la funcion
    let correo: String = env.get_string(&correo).unwrap().into();
    let pswd: String = env.get_string(&pswd).unwrap().into();
    if correo.is_empty() || pswd.is_empty() {
        env.call_method(
            this,
            "mostrar_error",
            "(Ljava/lang/String;)V", //objeto, fn name, parametros y tipo de retorno de la fn
            &[JValue::from(
                &env.new_string("Correo o contraseña vacíos").unwrap(),
            )],
        )
        .unwrap(); //argumentos, necesita ser JValue y un array
        return;
    } else {
        let runtime = TOKIO_RUNTIME.get_or_init(|| Runtime::new().unwrap());
        let this_ref = env.new_global_ref(this).unwrap();
        let client = Arc::new(reqwest::Client::new());
        runtime.spawn(verificar_credenciales(this_ref, client, correo, pswd));
    }
}

//funcion para registrar un usuario en la BD
#[unsafe(no_mangle)]
pub extern "C" fn Java_com_example_faena_register_registrarUsuario(
    mut env: JNIEnv,
    this: JObject,
    username: JString,
    correo: JString,
    pswd: JString,
    confirm_pswd: JString,
) {
    let correo: String = env.get_string(&correo).unwrap().into();
    let pswd: String = env.get_string(&pswd).unwrap().into();
    let username: String = env.get_string(&username).unwrap().into();
    let confirm_pswd: String = env.get_string(&confirm_pswd).unwrap().into();
    match validar_usuario(&username, &correo, &pswd, &confirm_pswd) {
        Ok(_) => {
            let runtime = TOKIO_RUNTIME.get_or_init(|| Runtime::new().unwrap());
            let this_ref = env.new_global_ref(this).unwrap();
            let client = Arc::new(reqwest::Client::new());
            runtime.spawn(registrar_usuario(this_ref, client, username, correo, pswd));
        }
        Err(err) => {
            env.call_method(
                &this,
                "mostrar_error",
                "(Ljava/lang/String;)V", //objeto, fn name, parametros y tipo de retorno de la fn
                &[JValue::from(&env.new_string(err.to_string()).unwrap())],
            )
            .unwrap(); //argumentos, necesita ser JValue y un array
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn Java_com_example_faena_P2PBridge_iniciarNodo(
    mut env: JNIEnv,
    this: JObject,
    direccion_remota: JString,
) {
    init_logger();

    let (tx, mut rx) = tokio::sync::mpsc::channel::<P2PMessage>(1);

    log::debug!("Obteniendo dirección remota...");
    let direccion_remota: Option<String> = if direccion_remota.is_null() {
        None
    } else {
        match env.get_string(&direccion_remota) {
            Ok(jstr) => {
                log::debug!("Dirección remota obtenida: {:?}", jstr.to_str());
                Some(jstr.into())
            }

            Err(e) => {
                log::error!("Error al obtener dirección remota: {e}");
                env.call_method(
                    &this,
                    "mostrar_error",
                    "(Ljava/lang/String;)V",
                    &[JValue::from(
                        &env.new_string(format!("Error al obtener dirección: {e}"))
                            .unwrap(),
                    )],
                )
                .unwrap();
                return;
            }
        }
    };

    let this_ref = match env.new_global_ref(&this) {
        Ok(global_ref) => global_ref,
        Err(e) => {
            env.call_method(
                this,
                "mostrar_error",
                "(Ljava/lang/String;)V",
                &[JValue::from(
                    &env.new_string(format!("Error creando referencia global: {e}"))
                        .unwrap(),
                )],
            )
            .unwrap();
            return; // Salir si no se puede crear la referencia global
        }
    };

    let runtime = TOKIO_RUNTIME.get_or_init(|| Runtime::new().unwrap());

    // Spawn para el nodo P2P
    runtime.spawn(async move {
        if let Err(e) = p2pcon::start_node(tx, direccion_remota).await {
            log::error!("Error en nodo P2P: {}", e);
        }
    });

    // Spawn separado para escuchar el canal
    runtime.spawn(async move {
        let jvm = JVM.get().expect("JVM no inicializada");

        loop {
            match timeout(Duration::from_secs(30), rx.recv()).await {
                Ok(Some(mensaje)) => {
                    let mut env = jvm.attach_current_thread().unwrap();

                    match mensaje {
                        P2PMessage::NewAddress(direccion) => {
                            let direccion_str = env.new_string(direccion.to_string()).unwrap();
                            env.call_method(
                                &this_ref,
                                "mostrarDireccion",
                                "(Ljava/lang/String;)V",
                                &[JValue::from(&direccion_str)],
                            )
                            .unwrap();
                        }
                        P2PMessage::PeerEvent(evento) => {
                            let evento_str = env.new_string(evento).unwrap();
                            env.call_method(
                                &this_ref,
                                "mostrarEvento", // Nuevo método que crearemos
                                "(Ljava/lang/String;)V",
                                &[JValue::from(&evento_str)],
                            )
                            .unwrap();
                        }
                    }
                }
                Ok(None) => break,
                Err(_) => continue,
            }
        }
    });
}

#[unsafe(no_mangle)]
pub extern "C" fn Java_com_example_faena_register_testJni(_env: JNIEnv, _class: JClass) -> jint {
    init_logger();
    #[cfg(target_os = "android")]
    log::info!("Rust: Función test_jni llamada exitosamente!");
    42
}

fn mostrar_error(err: String, this: &GlobalRef) {
    let jvm = JVM.get().expect("JVM sin inicializacion");
    let mut env = jvm.attach_current_thread().unwrap();
    let error_jstring = env.new_string(err).unwrap();
    env.call_method(
        &this,
        "mostrar_error",
        "(Ljava/lang/String;)V",
        &[JValue::from(&error_jstring)],
    )
    .expect("Fallo al mostrar error");
}
fn guardar_usuario(usuario: UsuarioGuardado, this: &GlobalRef) {
    let jvm = JVM.get().expect("JVM sin inicializacion");
    let mut env = jvm.attach_current_thread().unwrap();
    let nombre = env.new_string(&usuario.usuario).unwrap();
    let token = env.new_string(&usuario.token).unwrap();
    env.call_method(
        this,
        "guardar_usuario",
        "(Ljava/lang/String;Ljava/lang/String;)V",
        &[JValue::from(&nombre), JValue::from(&token)],
    )
    .unwrap();
}
