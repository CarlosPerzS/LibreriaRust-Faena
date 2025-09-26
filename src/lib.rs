//Archivos de la libreria
mod login;
mod modelo;
mod guardado_local;
mod crear_sala;
mod registrar;
mod archivos;

//Funciones dentro de los archivos externos a lib.rs
use registrar::{registrar_usuario, validar_usuario};
use guardado_local::recibir_id;
use crear_sala::validar_datos_sala_basica;
use login::verificar_credenciales;
use crate::crear_sala::{enviar_sala};
use crate::modelo::UsuarioGuardado;

//Las demas librerias o dependencias
use std::sync::Arc;
use chrono::{Utc, Duration, Local};
use jni::{JNIEnv, JavaVM};
use jni::objects::{GlobalRef,JClass, JObject, JValue, JString};
use jni::sys::{jint,JNI_VERSION_1_6};
use std::ffi::c_void;
use once_cell::sync::OnceCell;
use tokio::runtime::Runtime;

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
    let this_ref = env.new_global_ref(this).unwrap();
    if correo.is_empty() || pswd.is_empty(){
        mostrar_error("Correo o contraseña vacíos".to_string(), &this_ref);
        return;
    }
    else {
        let runtime = TOKIO_RUNTIME.get_or_init(|| Runtime::new().unwrap());
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
    let this_ref = env.new_global_ref(this).unwrap();
    match validar_usuario(&username, &correo, &pswd, &confirm_pswd){
        Ok(_) => {
            let runtime = TOKIO_RUNTIME.get_or_init(|| Runtime::new().unwrap());
            let client= Arc::new(reqwest::Client::new());
            runtime.spawn(registrar_usuario(this_ref,client, username, correo, pswd));
        }
        Err(err) => {
            mostrar_error(err.to_string(), &this_ref);
            return;
        }
    }
}

//funcion para crear una sala
#[unsafe(no_mangle)]
pub extern "C" fn Java_com_example_faena_createRoomBasic_crearSala(mut env: JNIEnv, this:JObject, nombre:JString, descripcion:JString, participantes:jint,acceso:JString, token:JString){
    //Asignaciones de java a rust
    let nombre_sala: String = env.get_string(&nombre).unwrap().into();
    let descripcion: String  = env.get_string(&descripcion).unwrap().into(); //ambos
    let num_participantes= participantes; //ambos
    let acceso:String = env.get_string(&acceso).unwrap().into();
    let mut is_privada: bool = false;
    let this_ref = env.new_global_ref(this).unwrap();
    //pasamos los valores de string de accesos y autorizacion a booleanos
    if acceso == "Privado".to_string(){
        is_privada = true;
    }
    //creamos las fechas de inicio y cierre de una sala basica
    let time_inicio = Utc::now().time().format("%H:%M:%S").to_string();
    let date_inicio = Utc::now().date_naive().format("%Y-%m-%d").to_string();
    let horas:i64 = 1; 
    //obtenemos el id de nuestro token JWT
    let mut id_usuario:i32 = 0; //id del usuario
    let id = recibir_id(env.get_string(&token).unwrap().into());
    match id {
        Ok(user_id) =>{ 
            id_usuario = user_id;
            mostrar_error(id_usuario.to_string(), &this_ref);
        }
        Err(error) => mostrar_error(error, &this_ref),
    }

    //Comprobamos primero si la sala basica es validad
    match validar_datos_sala_basica(&nombre_sala, &descripcion, num_participantes){
        //Si lo es, obtenemos el runtime y ejecutamos la accion de hacer post a la ruta en un hilo 
        Ok(_)=>{
            let runtime = TOKIO_RUNTIME.get_or_init(|| Runtime::new().unwrap());
            let client= Arc::new(reqwest::Client::new());
            runtime.spawn(enviar_sala(this_ref, client, nombre_sala, descripcion, num_participantes, is_privada,false, false, 
                date_inicio, time_inicio, horas, id_usuario));
        }
        Err(err)=>{
            mostrar_error(err.to_string(), &this_ref);
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn Java_com_example_faena_createRoomPremium_crearSala(mut env: JNIEnv, this:JObject, nombre:JString, descripcion:JString, participantes:jint,acceso:JString, autorizacion:JString, token:JString){
    //Asignaciones de java a rust
    let nombre_sala: String = env.get_string(&nombre).unwrap().into();
    let descripcion: String  = env.get_string(&descripcion).unwrap().into(); //ambos
    let num_participantes= participantes; //ambos
    let autorizacion: String = env.get_string(&autorizacion).unwrap().into();
    let acceso:String = env.get_string(&acceso).unwrap().into();
    let mut is_privada: bool = false;
    let mut is_filtro_dominio: bool = false;
    let this_ref = env.new_global_ref(this).unwrap();
    //pasamos los valores de string de accesos y autorizacion a booleanos
    if acceso == "Privado".to_string(){
        is_privada = true;
    }
    if autorizacion == "Dominios de Correos".to_string(){
        is_filtro_dominio = true; 
    }
    //creamos las fechas de inicio y cierre de una sala basica
    let time_inicio = (Utc::now().time() + Duration::seconds(20)).to_string();
    let date_inicio = Utc::now().date_naive().to_string();
    let horas:i64 = 1; 
    //obtenemos el id de nuestro token JWT
    let mut id_usuario:i32 = 0; //id del usuario
    let id = recibir_id(env.get_string(&token).unwrap().into());
    match id {
        Ok(user_id) => id_usuario = user_id,
        Err(error) => println!("Error obteniendo ID: {}", error),
    }

    //Comprobamos primero si la sala basica es validad
    match validar_datos_sala_basica(&nombre_sala, &descripcion, num_participantes){
        //Si lo es, obtenemos el runtime y ejecutamos la accion de hacer post a la ruta en un hilo 
        Ok(_)=>{
            let runtime = TOKIO_RUNTIME.get_or_init(|| Runtime::new().unwrap());
            let client= Arc::new(reqwest::Client::new());
            runtime.spawn(enviar_sala(this_ref, client, nombre_sala, descripcion, num_participantes, is_privada,is_filtro_dominio, false, 
                date_inicio, time_inicio, horas, id_usuario));
        }
        Err(err)=>{
            mostrar_error(err.to_string(), &this_ref);
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

//Funciones generales para ejecutar en java

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
fn sala_creada(this: &GlobalRef){
    let jvm = JVM.get().expect("JVM sin inicializacion");
    let mut env = jvm.attach_current_thread().unwrap();
    env.call_method(this, "sala_creada", "()V", &[]).unwrap();
}