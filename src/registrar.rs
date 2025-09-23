use crate::modelo::{UsuarioGuardado, NuevoUsuario};
use crate::{mostrar_error, guardar_usuario};
use jni::objects::GlobalRef;
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

//Funciones para validar el correo
fn correo_valido(email:&str) -> bool{
    if email.contains("@"){
        if let Some(dominio) = email.split('@').nth(1) {
            if dominio.contains("."){
                true
            }
            else{
                false
            }
        } else {
            false 
        }
    }
    else{
        false
    }
}

// Funcion para validar el usuario que quiere registrar la persona
pub fn validar_usuario(username: &str, correo: &str, pswd: &str, confirm_pswd: &str) -> Result<(), &'static str>{
    if correo.is_empty() || username.is_empty() || pswd.is_empty() || confirm_pswd.is_empty(){
        Err("No puede haber campos vacíos")
    }
    else if !correo_valido(correo){
        Err("El correo no es valido")
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

pub async fn registrar_usuario(this: GlobalRef,cliente: Arc<Client>,username: String,correo: String,password: String){
    let nuevo_usuario= NuevoUsuario{
        nombre: username.clone(),
        email: correo.clone(),
        contrasena: password.clone(),
    };
    let url="http://192.168.100.76:8001/api/usuarios";
    let res= cliente.post(url).json(&nuevo_usuario).send().await;
    match res {
        Ok(_res)=>{
            if _res.status().is_success(){
                match _res.json::<UsuarioGuardado>().await {
                    Ok(usuario) => {
                        guardar_usuario(usuario, &this);
                    }
                    Err(err)=>{
                        mostrar_error(err.to_string(), &this);
                    }
                }
            }
            else if _res.status()==409 {
                mostrar_error("Correo electronico ya registrado".to_string(), &this);
            }
            else{
                let error_texto = _res.text().await.unwrap_or_default();
                mostrar_error(error_texto, &this);
            }
        }
        Err(err)=>{
            mostrar_error(err.to_string(), &this);
        }
    }
}