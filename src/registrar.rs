use crate::modelo::{Usuario, UsuarioGuardado};
use jni::objects::JValue;
use jni::JNIEnv;
use jni::objects::{JObject};
use reqwest::Client;
use std::sync::Arc;

// Funciones para validar la password
fn buscar_caracter_especial(password:&str) -> bool{
    password.chars().any(|letra| !letra.is_ascii_alphanumeric())
}
fn buscar_caracter_digito(password: &str) -> bool{
    password.chars().any(|letra| letra.is_ascii_digit())
}
fn buscar_caracter_mayuscula(password: &str) -> bool{
    password.chars().any(|letra| letra.is_ascii_uppercase())
}
fn buscar_caracter_minuscula(password: &str) -> bool{
    password.chars().any(|letra| letra.is_ascii_lowercase())
}

// Funcion para validar el usuario que quiere registrar la persona
pub fn validar_usuario(username: &str, correo: &str, pswd: &str, confirm_pswd: &str) -> Result<(), &'static str>{
    if correo.is_empty() || username.is_empty() || pswd.is_empty() || confirm_pswd.is_empty(){
        Err("No puede haber campos vacíos")
    }
    else if pswd != confirm_pswd{
        Err("Las contraseñas no coinciden")
    }
    else if pswd.chars().count() < 8{
        Err("La contraseña necesita ser de minimo 8 caracteres")
    }
    else if !buscar_caracter_especial(pswd) {
        Err("La contraseña no tiene minimo 1 caracter especial")
    }
    else if !buscar_caracter_digito(pswd) {
        Err("La contraseña no tiene minimo 1 digito")
    }
    else if !buscar_caracter_mayuscula(pswd) {
        Err("La contraseña no tiene minimo 1 mayuscula")
    }
    else if !buscar_caracter_minuscula(pswd) {
        Err("La contraseña no tiene minimo 1 minuscula")
    }
    else{
        Ok(())
    }
}

pub async fn registrar_usuario(mut env: JNIEnv<'_>, this: JObject<'_>,cliente: Arc<Client>,username: String,correo: String,password: String){
    let nuevo_usuario= Usuario{
        nombre: username.clone(),
        email: correo.clone(),
        contrasena: password.clone(),
        premium: false,
    };
    let url="http://192.168.100.76:8001/usuarios";
    let res= cliente.post(url).json(&nuevo_usuario).send().await;
    match res {
        Ok(_res)=>{
            if _res.status().is_success(){
                match _res.json::<UsuarioGuardado>().await {
                    Ok(usuario) => {
                        let nombre = env.new_string(&usuario.usuario.nombre).unwrap();
                        let token = env.new_string(&usuario.token).unwrap();
                        env.call_method(&this, "guardar_usuario", "(Ljava/lang/String;Ljava/lang/String;)V",
                        &[JValue::from(&nombre), JValue::from(&token)]).unwrap();
                    }
                    Err(err)=>{
                        eprintln!("Error al parsear JSON: {:?}", err);
                    }
                }
            }
            else{
                eprintln!("ERROR: {:?}", _res);
            }
        }
        Err(err)=>{
            eprintln!("Fallo al hacer peticion POST: {:?}", err);
        }
    }
}