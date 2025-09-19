#[allow(dead_code)]
mod login;
mod modelo;
mod p2pcon;
#[allow(dead_code)]
mod registrar;
#[allow(unused_imports)]
use registrar::{registrar_usuario, validar_usuario};
#[allow(unused_imports)]
use std::sync::Arc;
use login::verificar_credenciales;
use jni::JNIEnv;
use jni::objects::{JClass, JObject, JValue, JString};
use jni::sys::{jint};

use futures::prelude::*;
use p2pcon::init_node;

use crate::modelo::Usuario;

static LOGGER_INIT: std::sync::Once = std::sync::Once::new();

fn init_logger() {
    #[cfg(target_os = "android")]
    {
        LOGGER_INIT.call_once(|| {
            android_logger::init_once(
                android_logger::Config::default()
                    .with_max_level(log::LevelFilter::Debug)
                    .with_tag("Rust")
            );
        });
    }
}
    //se necesita que el nombre sea Java_paquete_clase_nombre de la funcion
//funcion para iniciar sesion en la bd desde rust
pub extern "C" fn Java_com_example_faena_login_login(mut env: JNIEnv, this:JObject, correo:JString, pswd:JString){ //en env y class/this son argumentos dados por JNI cuando se invoca a la funcion
    let correo: String = env.get_string(&correo).unwrap().into();
    let pswd: String = env.get_string(&pswd).unwrap().into();
    if correo.is_empty() || pswd.is_empty(){
        env.call_method(this, "mostrar_error", "(Ljava/lang/String;)V", //objeto, fn name, parametros y tipo de retorno de la fn
        &[JValue::from(&env.new_string("Correo o contraseña vacíos").unwrap())]).unwrap(); //argumentos, necesita ser JValue y un array
        return;
    } 
    else {
        let client = Arc::new(reqwest::Client::new());
        let runtime = tokio::runtime::Runtime::new().unwrap();
        runtime.block_on(verificar_credenciales(env, this,client, correo, pswd));
    }
}

//funcion para registrar un usuario en la BD
pub extern "C" fn Java_com_example_faena_register_registrarUsuario(mut env: JNIEnv, this:JObject, username:JString, correo:JString, pswd:JString,confirm_pswd:JString){
    let correo: String = env.get_string(&correo).unwrap().into();
    let pswd: String = env.get_string(&pswd).unwrap().into();
    let username: String = env.get_string(&username).unwrap().into();
    let confirm_pswd: String = env.get_string(&confirm_pswd).unwrap().into();
    match validar_usuario(&username, &correo, &pswd, &confirm_pswd){
        Ok(_) => {
            let client= Arc::new(reqwest::Client::new());
            let runtime = tokio::runtime::Runtime::new().unwrap();
            runtime.block_on(registrar_usuario(env, this,client, username, correo, pswd));
        }
        Err(err) => {
            env.call_method(this, "mostrar_error", "(Ljava/lang/String;)V", //objeto, fn name, parametros y tipo de retorno de la fn
        &[JValue::from(&env.new_string(err.to_string()).unwrap())]).unwrap(); //argumentos, necesita ser JValue y un array
        }
    }
}

pub extern "C" fn Java_com_example_faena_P2PBridge_iniciarNodo(mut env: JNIEnv, this: JObject) {
   
    init_logger();

    let (tx, mut rx) = tokio::sync::mpsc::channel(1);

    let runtime = tokio::runtime::Runtime::new().unwrap();

    // Ejecuta el nodo y el loop de eventos en el runtime
    runtime.block_on(async {
        // Inicializa el nodo
        let mut swarm = match init_node(tx).await {
            Ok(s) => s,
            Err(e) => {
                
                env.call_method(
                    this,
                    "mostrar_error",
                    "(Ljava/lang/String;)V",
                    &[JValue::from(&env.new_string(format!("Error al iniciar nodo: {e}")).unwrap())]
                ).unwrap();
                return;
            }
        };

        // Loop de eventos principal
        loop {
            match swarm.select_next_some().await {
                libp2p::swarm::SwarmEvent::NewListenAddr { address, .. } => {
                    // Aquí podrías enviar la dirección a Java si lo necesitas
                }
                libp2p::swarm::SwarmEvent::Behaviour(event) => {
                    // Aquí podrías procesar eventos de ping, etc.
                }
                _ => {}
            }
        }
    });
}



#[unsafe(no_mangle)]
pub extern "C" fn Java_com_example_faena_register_testJni(
    _env: JNIEnv,
    _class: JClass,
) -> jint {
    init_logger();
    #[cfg(target_os = "android")]
    log::info!("🦀 Rust: Función test_jni llamada exitosamente!");
    42
}