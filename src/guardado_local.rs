use jsonwebtoken::{decode, DecodingKey, Validation, Algorithm};
use crate::modelo::Claims; //modelos de usuarios


pub fn recibir_id(token:String) -> Result<i32, String> {
    let jwt = token.clone(); //el token encoded
    // configurar validación
    let validation = Validation::new(Algorithm::HS256);
    //hacemos la decodificacion de el token, basicamente lo contrario a lo que se hace en fn generar_jwt
    match decode::<Claims>(&jwt, &DecodingKey::from_secret(b"clabe"), &validation,){
        Ok(token) =>{
            Ok(token.claims.sub)
        }
        Err(e) => {
            Err(format!("JWT inválido: {}", e))
        }
    }        
}