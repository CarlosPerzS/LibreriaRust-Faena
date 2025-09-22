#[allow(dead_code)]
mod login;
mod modelo;
#[allow(dead_code)]
mod registrar;
#[allow(unused_imports)]
use registrar::{registrar_usuario, validar_usuario};
#[allow(unused_imports)]
use std::sync::Arc;
use login::verificar_credenciales;
use jni::{JNIEnv, JavaVM};
use jni::objects::{GlobalRef,JClass, JObject, JValue, JString};
use jni::sys::{jint,JNI_VERSION_1_6};
use std::ffi::c_void;
use once_cell::sync::OnceCell;
use tokio::runtime::Runtime;

use crate::modelo::UsuarioGuardado;

//variables publicas para la conexion entre la app y libreria
pub static TOKIO_RUNTIME: OnceCell<Runtime> = OnceCell::new(); //gestor de hilos principal
pub static JVM: OnceCell<JavaVM> = OnceCell::new(); //jvm de parte de android, las almacenamos en OnceCell para evitar cambios repentinos
static LOGGER_INIT: std::sync::Once = std::sync::Once::new();

//funcion para inicializar la variable publica de JVM para poder tener tareas asincronas como las peticiones reqwest e inicializacion en general
#[unsafe(no_mangle)] 
pub extern "system" fn JNI_OnLoad(vm: JavaVM, _: *mut c_void) -> jint { //recibimos la JVM de java android
    init_logger(); // 
    TOKIO_RUNTIME.get_or_init(|| Runtime::new().unwrap()); //incializacion de runtime de tokio
    if JVM.set(vm).is_err() { //inicializacion de la jvm (variable global de la libreria)
        return -1; // JNI_ERR
    }
    JNI_VERSION_1_6
}

fn init_logger() {
    #[cfg(target_os = "android")]
    {
        LOGGER_INIT.call_once(|| {
            android_logger::init_once(
                android_logger::Config::default().with_max_level(log::LevelFilter::Debug).with_tag("Rust")
            );
        });
    }
}
    //se necesita que el nombre sea Java_paquete_clase_nombre de la funcion
//funcion para iniciar sesion en la bd desde rust
#[unsafe(no_mangle)]
pub extern "C" fn Java_com_example_faena_login_login(mut env: JNIEnv, this:JObject, correo:JString, pswd:JString){ //en env y class/this son argumentos dados por JNI cuando se invoca a la funcion
    let correo: String = env.get_string(&correo).unwrap().into();
    let pswd: String = env.get_string(&pswd).unwrap().into();
    if correo.is_empty() || pswd.is_empty(){
        env.call_method(this, "mostrar_error", "(Ljava/lang/String;)V", //objeto, fn name, parametros y tipo de retorno de la fn
        &[JValue::from(&env.new_string("Correo o contraseña vacíos").unwrap())]).unwrap(); //argumentos, necesita ser JValue y un array
        return;
    }
    else {
        let runtime = TOKIO_RUNTIME.get_or_init(|| Runtime::new().unwrap());
        let this_ref = env.new_global_ref(this).unwrap();
        let client= Arc::new(reqwest::Client::new());
        runtime.spawn(verificar_credenciales(this_ref,client, correo, pswd));
    }
}

//funcion para registrar un usuario en la BD
#[unsafe(no_mangle)]
pub extern "C" fn Java_com_example_faena_register_registrarUsuario(mut env: JNIEnv, this:JObject, username:JString, correo:JString, pswd:JString,confirm_pswd:JString){
    let correo: String = env.get_string(&correo).unwrap().into();
    let pswd: String = env.get_string(&pswd).unwrap().into();
    let username: String = env.get_string(&username).unwrap().into();
    let confirm_pswd: String = env.get_string(&confirm_pswd).unwrap().into();
    match validar_usuario(&username, &correo, &pswd, &confirm_pswd){
        Ok(_) => {
            let runtime = TOKIO_RUNTIME.get_or_init(|| Runtime::new().unwrap());
            let this_ref = env.new_global_ref(this).unwrap();
            let client= Arc::new(reqwest::Client::new());
            runtime.spawn(registrar_usuario(this_ref,client, username, correo, pswd));
        }
        Err(err) => {
            env.call_method(&this, "mostrar_error", "(Ljava/lang/String;)V", //objeto, fn name, parametros y tipo de retorno de la fn
        &[JValue::from(&env.new_string(err.to_string()).unwrap())]).unwrap(); //argumentos, necesita ser JValue y un array
        }
    }
}


#[unsafe(no_mangle)]
pub extern "C" fn Java_com_example_faena_register_testJni(
    _env: JNIEnv,
    _class: JClass,
) -> jint {
    init_logger();
    #[cfg(target_os = "android")]
    log::info!("Rust: Función test_jni llamada exitosamente!");
    42
}

fn mostrar_error(err:String, this: &GlobalRef){
    let jvm = JVM.get().expect("JVM sin inicializacion");
    let mut env = jvm.attach_current_thread().unwrap();
    let error_jstring = env.new_string(err).unwrap();
    env.call_method(&this,"mostrar_error","(Ljava/lang/String;)V",
    &[JValue::from(&error_jstring)],).expect("Fallo al mostrar error");
}
fn guardar_usuario(usuario:UsuarioGuardado, this: &GlobalRef){
    let jvm = JVM.get().expect("JVM sin inicializacion");
    let mut env = jvm.attach_current_thread().unwrap();
    let nombre = env.new_string(&usuario.usuario).unwrap();
    let token = env.new_string(&usuario.token).unwrap();
    env.call_method(this, "guardar_usuario", "(Ljava/lang/String;Ljava/lang/String;)V",
    &[JValue::from(&nombre), JValue::from(&token)]).unwrap();
}