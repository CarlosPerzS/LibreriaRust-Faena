#[allow(dead_code)]
mod login;
mod modelo;
#[allow(unused_imports)]
use std::sync::Arc;
use login::verificar_credenciales;
use jni::JNIEnv;
use jni::objects::{JClass, JObject, JValue, JString};
use jni::sys::{jint};

static LOGGER_INIT: std::sync::Once = std::sync::Once::new();

fn init_logger() {
    #[cfg(target_os = "android")]
    {
        LOGGER_INIT.call_once(|| {
            android_logger::init_once(
                android_logger::Config::default()
                    .with_max_level(log::LevelFilter::Debug)
                    .with_tag("RustSlint")
            );
        });
    }
}
    //se necesita que el nombre sea Java_paquete_clase_nombre de la funcion
pub extern "C" fn Java_com_example_faena_MainActivity_login(mut env: JNIEnv, this:JObject, correo:JString, pswd:JString){ //en env y class/this son argumentos dados por JNI cuando se invoca a la funcion
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


#[unsafe(no_mangle)]
pub extern "C" fn Java_com_example_faenaapp_MainActivity_test_1jni(
    _env: JNIEnv,
    _class: JClass,
) -> jint {
    init_logger();
    #[cfg(target_os = "android")]
    log::info!("🦀 Rust: Función test_jni llamada exitosamente!");
    42
}