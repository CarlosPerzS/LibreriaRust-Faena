use crate::modelo::{Credenciales, UsuarioGuardado};
use crate::JVM;
use jni::objects::{GlobalRef, JValue};
use reqwest::Client;
use std::sync::Arc;
use crate::mostrar_error;

pub async fn verificar_credenciales(this: GlobalRef, cliente: Arc<Client>,correo: String,password: String) 
{   
    let jvm = JVM.get().expect("Error al conectar con JVM");
    let url = "http://192.168.100.76:8001/api/login"; // URL de la API
    let credenciales = Credenciales { //creamos una instancia de la clase credenciales con los datos de login
        email: correo,
        contrasena: password,
    };

    match cliente.post(url).json(&credenciales).send().await { //hacemos la peticion de login
        //si la respuesta no es un error
        Ok(res) => {
            let status = res.status(); //status de la respuesta de /login
            //si la peticion esta correcta y se regresa un json
            if status.is_success() {
                match res.json::<UsuarioGuardado>().await { //parseamos la respuesta de json a nuestra clase usuario
                    Ok(usuario) => {
                        let mut env = jvm.attach_current_thread().unwrap();
                        let nombre = env.new_string(&usuario.usuario).unwrap();
                        let token = env.new_string(&usuario.token).unwrap();
                        env.call_method(this, "guardar_usuario", "(Ljava/lang/String;Ljava/lang/String;)V",
                        &[JValue::from(&nombre), JValue::from(&token)]).unwrap();
                    }
                    Err(err) => {
                        eprintln!("Error al parsear JSON: {:?}", err);
                        mostrar_error(err.to_string(), &this);
                    }
                }
            } else if status == reqwest::StatusCode::UNAUTHORIZED { //revisamos si el status fue un error (no existen esas credenciales)
                mostrar_error("Credenciales no validas".to_string(), &this);
            }
        }
        //en caso de que exista un error en el proceso de la peticion
        Err(err) => {
            mostrar_error(err.to_string(), &this);
        }
    }
}
