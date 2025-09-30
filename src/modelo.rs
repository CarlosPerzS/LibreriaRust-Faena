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
pub struct NuevoUsuario {
    pub nombre: String,
    pub contrasena: String,
    pub email: String
}

#[derive(Debug, Deserialize, Serialize, Default)]
// estructura de los usuarios en rust, posteriolmente se convierte en json
pub struct UsuarioGuardado {
    pub usuario: String,
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

#[derive(Serialize)]
pub struct SalaVotacion {
    pub id: i32,
    pub nombre: String,
    pub descripcion: String,
    pub recurrente: bool,
    pub privada: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filtro_dominio: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub codigo_acceso: Option<String>,
    pub max_participantes: i32,
    pub fecha_inicio: Option<chrono::NaiveDate>,
    pub hora_inicio: Option<chrono::NaiveTime>,
    pub hora_cierre: Option<chrono::NaiveTime>,
    pub creador_id: i32,
    pub creado_en: chrono::NaiveDateTime,
    pub activa: bool,
}

#[derive(Deserialize, Clone)]
pub struct NuevoFiltroSalaRequest {
    pub sala_id: i32,
    pub valores: Vec<String>
}

#[derive(Serialize)]
pub struct NuevoFiltroSala {
    pub sala_id: i32,
    pub valor: String
}

#[derive(Serialize)]
pub struct SalaFiltro {
    pub id: i32,
    pub sala_id: i32,
    pub valor: String
}

#[derive(Serialize)]
pub struct FiltrosTodosResponse {
    pub filtros: Vec<SalaFiltro>,
    pub total: usize,
    pub tipo_filtro: String,
}