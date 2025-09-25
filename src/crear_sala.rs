use chrono::{DateTime, Local, NaiveDate, NaiveTime, Duration};
use reqwest::Client;
use core::time;
use std::sync::Arc;
use jni::objects::GlobalRef;
use crate::{sala_creada, mostrar_error};
use crate::modelo::{NuevaSalaVotacion};
//establecemos DateTime en caso de que el usuario deje vacio el campo, le ponemos un minuto despues para evitar problemas

//calculamos la hora de cierre en base a los dos datos que tenemos
//pub fn hora_de_cierre(hora_inicio:&str, duracion_sala: i32) -> String{}

pub fn establecer_hora_predeterminada(hora_input: &str) -> String {
    if hora_input.is_empty() {
        let now = Local::now();
        let hora_actual = now.time() + Duration::minutes(1); //se agrega un minuto
        hora_actual.format("%H:%M:%S").to_string()
    } else {
        hora_input.to_string()
    }
}

pub fn establecer_fecha_predeterminada(fecha_input: &str) -> String {
    if fecha_input.is_empty() {
        let now = Local::now();
        let fecha_actual = now.date_naive();
        fecha_actual.format("%Y-%m-%d").to_string()
    } else {
        fecha_input.to_string()
    }
}

pub fn verificar_date_inicio(date_inicio:&str, hoy:DateTime<Local>) -> bool{
    let fecha_actual = hoy.date_naive(); //de la funcion now obtenemos la fecha, en este caso, la que manipulamos
    let fecha_inicio = match NaiveDate::parse_from_str(date_inicio, "%Y-%m-%d") { //pasrseamos la fecha de str a Date con un formato especifico, que es el standart
        Ok(fecha) => fecha, //tenemos que hacer match para cuidar de errores en caso de que no se pueda parsear
        Err(err) => {
            println!("Error en parsear: {}", err);
            return false;
        }
    };
    //si la fecha es mayor o igual a la actual retorna true, si es menor, false
    fecha_inicio >= fecha_actual
}

pub fn verificar_time_inicio(time_inicio:&str, date_inicio:&str, hoy:DateTime<Local>) -> bool{
    let hora_actual= hoy.time(); //hora actual del dispositivo
    let fecha_actual=hoy.date_naive();
    //parseamos la hora 
    let hora_inicio= match NaiveTime::parse_from_str(time_inicio, "%H:%M:%S"){ //parseamos la hora al formato especifico
        Ok(hora)=>hora,
        Err(err) => {
            println!("Error en parsear: {}", err);
            return false;
        }
    };
    //verificamos si la fecha es la del dia actual, si no lo es cualquier hora parseada correctamente es valida
    let fecha_inicio = match NaiveDate::parse_from_str(date_inicio, "%Y-%m-%d") { //pasrseamos la fecha de str a Date con un formato especifico, que es el standart
        Ok(fecha) => fecha, //tenemos que hacer match para cuidar de errores en caso de que no se pueda parsear
        Err(err) => {
            println!("Error en parsear: {}", err);
            return false;
        }
    };
    if fecha_inicio == fecha_actual{
        //si la hora es mayor o igual a la actual retorna true, si es menor, false
        hora_inicio >= hora_actual
    }
    else {
        true
    }
}


//VALIDACIONES POR SALA

pub fn validar_datos_sala_basica(nombre: &str, descripcion: &str, participantes: i32) ->  Result<(), &'static str>{
    //verificamos los posibles errores que pueda tener el usuario al registrar una sala basica
    if nombre.trim().is_empty() || descripcion.trim().is_empty(){ 
        Err("No puede haber campos vacíos")
    }
    else if participantes > 100 || participantes < 1{
        Err("Numero de participantes debe de ser entre 1 y 100")        
    }
    else { //si no hay errores retornamos sin error
        Ok(())
    }
}

pub fn validar_datos_sala_premium (nombre: &str, descripcion: &str, participantes: i32,date_inicio:&str, time_inicio:&str,
duracion_sala: i32, is_recurrente:bool) -> Result<(), &'static str>{
    let now = Local::now(); //agarramos la hora y tiempo local del dispositivo
    //verificamos sala de premiums
    if nombre.is_empty() || descripcion.is_empty(){
        Err("No puede haber campos vacios")
    }
    else if participantes > 1000 || participantes < 1 {
        Err("Numero de participantes debe de ser entre 1 y 1000")
    }
    else if !is_recurrente && (duracion_sala > 24 && duracion_sala < 1) {
        Err("Numero de horas debe de ser entre 1 y 24")
    }
    else if !is_recurrente && !verificar_date_inicio(date_inicio, now){ //se hace un NOT de la funcion pq retorna true si es valido
        Err("La fecha de inicio no puede ser pasada")
    }
    else if !is_recurrente && !verificar_time_inicio(time_inicio, date_inicio, now) {
        Err("La hora de inicio no puede ser pasada")
    }
    else {
        Ok(())
    }
}

//ENVIAR SALA A LA BD
pub async fn enviar_sala(
    this: GlobalRef,
    cliente: Arc<Client>,
    nombre: String,
    descripcion: String,
    num_participantes: i32,
    is_privada:bool,
    is_filtro_dominio:bool,
    is_recurrente:bool,
    date_inicio:String,
    time_inicio:String,
    duracion_sala: i64, 
    id:i32
) -> Result<(), &'static str>{
    //Declaramos las variables option que son las de chronos
    let hora_inicio:Option<NaiveTime>;
    let fecha_inicio:Option<NaiveDate>;
    let hora_cierre:Option<NaiveTime>;
    //Manipulamos los tiempos si es unica, si es recurrente las ponemos None
    if is_recurrente{
        hora_inicio = None;
        fecha_inicio = None;
        hora_cierre = None;
    }
    else{
        let hora_parseada = NaiveTime::parse_from_str(&time_inicio, "%H:%M:%S").expect("Error al parsear la hora");
        hora_inicio = Some(hora_parseada + Duration::seconds(20));
        fecha_inicio = Some(NaiveDate::parse_from_str(&date_inicio, "%Y-%m-%d").expect("Error al parsear la fecha"));
        hora_cierre = Some(hora_parseada + Duration::hours(duracion_sala));
    }
    let url = "http://192.168.100.76:8001/api/salas_votacion";
    let sala = NuevaSalaVotacion{
        nombre:nombre,
        descripcion:descripcion,
        recurrente:is_recurrente,
        privada:is_privada,
        max_participantes:num_participantes,
        fecha_inicio:fecha_inicio,
        hora_inicio:hora_inicio,
        hora_cierre:hora_cierre,
        creador_id:id,
        filtro_dominio:Some(is_filtro_dominio),
        codigo_acceso:Some("".to_string()),
        activa:Some(true)
    };
    match cliente.post(url).json(&sala).send().await {
        Ok(res) => {
            let status = res.status();
            if status == reqwest::StatusCode::OK{
                sala_creada(&this);
            }
        }
        Err(err)=>{
            println!("Error al hacer peticion a la API: {}", err);
            mostrar_error(err.to_string(), &this);
        }
    }
    Ok(())
}