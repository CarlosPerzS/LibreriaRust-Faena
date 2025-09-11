use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Default)]
// estructura de los usuarios en rust, posteriolmente se convierte en json
pub struct Usuario {
    pub nombre: String,
    pub contrasena: String,
    pub email: String,
    pub premium: bool,
}

#[derive(Debug, Deserialize, Serialize, Default)]
// estructura de los usuarios en rust, posteriolmente se convierte en json
pub struct UsuarioGuardado {
    pub usuario: Usuario,
    pub token: String,
}

#[derive(Serialize)]
pub struct NuevaSalaVotacion {
    pub nombre: String,
    pub descripcion: String,
    pub recurrente: bool,
    pub privada: bool,
    pub filtro_dominio: Option<bool>, 
    pub codigo_acceso: Option<String>, 
    pub max_participantes: i32,
    pub fecha_inicio: Option<chrono::NaiveDate>,
    pub hora_inicio: Option<chrono::NaiveTime>,
    pub hora_cierre: Option<chrono::NaiveTime>,
    pub creador_id: i32,
    pub activa: Option<bool>, 
}

#[derive(Deserialize, Serialize)]
pub struct Credenciales {
    pub email: String,
    pub contrasena: String,
}

//Estructura del JWT
#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: i32,
    pub email: String,
    pub exp: usize,
}