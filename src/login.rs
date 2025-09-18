use crate::modelo::{Credenciales, UsuarioGuardado};
use crate::guardado_local::{guardar_usuario, cargar_usuario};
use jni::objects::JValue;
use reqwest::Client;
use std::sync::Arc;

pub async fn verificar_credenciales(mut env: JNIEnv, this:JObject, cliente: Arc<Client>,correo: String,password: String) 
{
    let url = "http://192.168.137.12:8001/api/login"; // URL de la API
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
                        env.call_method(this, "guardar_usuario", "(Ljava/lang/UsuarioGuardado;)V",
                        &[JValue::from(env.new_class(usuario).unwrap())]).unwrap()
                    }
                    Err(err) => {
                        eprintln!("Error al parsear JSON: {:?}", err);
                    }
                }
            } else if status == reqwest::StatusCode::UNAUTHORIZED { //revisamos si el status fue un error (no existen esas credenciales)
                env.call_method(this, "mostrar_error", "(Ljava/lang/String;)V",
                &[JValue::from(env.new_string("Credenciales no validas").unwrap())]).unwrap();
            }
        }
        //en caso de que exista un error en el proceso de la peticion
        Err(err) => {
            env.call_method(this, "mostrar_error", "(Ljava/lang/String;)V",
            &[JValue::from(env.new_string("Error al realizar la peticion").unwrap())]).unwrap();
        }
    }
}