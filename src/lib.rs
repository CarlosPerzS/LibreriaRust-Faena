

#[allow(dead_code)]
mod login;
#[allow(unused_imports)]
use login::verificar_credenciales;

use jni::JNIEnv;
use jni::objects::{JClass, JObject, GlobalRef};
use jni::sys::{jint, jlong};

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
    let correo = env.get_string(correo).unwrap().into();
    let pswd = env.get_string(pswd).unwrap().into();
    if correo.is_empty() || pswd.is_empty(){
        env.call_method(this, "mostrar_error", "(Ljava/lang/String;)V", //objeto, fn name, parametros y tipo de retorno de la fn
        &[JValue::from(env.new_string("Correo o contraseña vacíos").unwrap())]).unwrap(); //argumentos, necesita ser JValue y un array
        return;
    } 
    else {
        let client = Arc::new(reqwest::Client::new());
        tokio::spawn(verificar_credenciales(env, this,client, correo, pswd));
    }
}
fn start_rust(ui: &MainWindow) {
    #[cfg(target_os = "android")]
    log::info!("🦀 Rust: Configurando lógica de la app...");
    let salas_model = Rc::new(VecModel::from(Vec::<(SharedString, SharedString)>::new()));
    ui.set_salas_admin(ModelRc::from(salas_model.clone()));
    let salas_model_add = salas_model.clone();

    let ui_weak = ui.as_weak();

    let client = Arc::new(reqwest::Client::new());  //cliente http que provee reqwest
    let client1 = client.clone();
    let client2 = client.clone();
    let client3 = client.clone();
    
     //conocer si hay una sesion activa
    if let Some(usuario) = cargar_usuario() {
        println!("Sesión encontrada: {}", usuario.usuario.nombre);
        ui.set_user(usuario.usuario.nombre.into());
        ui.set_pantalla_actual(3); // Ir directo al menú
    } else {
        println!("No hay sesión activa");
        ui.set_pantalla_actual(0); // Pantalla de login
    }
    //Boton de registrar
    ui.on_registrar({
        let ui_handle = ui.as_weak();
        move || {
            let ui_instance = ui_handle.unwrap(); 
            let correo = ui_instance.get_email_registro().to_string(); 
            let pswd = ui_instance.get_pswd_registro().to_string();
            let username = ui_instance.get_username().to_string();
            let confirm_pswd = ui_instance.get_confirm_pswd().to_string();

            match validar_usuario(&username, &correo, &pswd, &confirm_pswd){
                Ok(_) => {
                    let client= client2.clone();
                    tokio::spawn(registrar_usuario(client, username, correo, pswd, ui_handle.clone()));
                }
                Err(err) => {
                    ui_instance.set_empty_error(err.into());
                    ui_instance.set_empty_error_text_color(ui_instance.get_rojo());
                    ui_instance.set_input_border_color(ui_instance.get_border());
                }
            }
        }
    });

    //Boton de cerrar sesion
    ui.on_logout({
        let ui_handle = ui.as_weak();
        move ||{
            let ui_instance = ui_handle.unwrap(); 
            eliminar_usuario();
            ui_instance.set_pantalla_actual(1);
            ui_instance.set_email_login("".into());
            ui_instance.set_pswd_login("".into());       
        }
    });

    //Boton de enviar codigo para unirse a sala
    ui.on_enviar_codigo({
        let ui_handle = ui.as_weak();
        move || {
            let ui_instance = ui_handle.unwrap();
            let codigo = ui_instance.get_codigo().to_string();
            if codigo.chars().count() != 6{
                ui_instance.set_empty_error("El código debe de llevar 6 caracteres".into());
                ui_instance.set_empty_error_text_color(ui_instance.get_rojo());
            }
            else {
                println!("entrando sala...");
            }
        }
    });

    //Boton para ir a crear sala de votacion
    ui.on_verificar_premium({
        let ui_handle = ui.as_weak();
        move || {
            let ui_instance = ui_handle.unwrap();
            if let Some(usuario)=cargar_usuario(){
                if usuario.usuario.premium{
                    ui_instance.set_pantalla_actual(7);
                }
                else{
                    ui_instance.set_pantalla_actual(6);
                }
            }
        }
    });

    //Boton para crear sala de votacion
    ui.on_crear_sala({
        let ui_handle = ui.as_weak();
        move || {
            let ui_instance = ui_handle.unwrap();
            //datos de la salas y si son para ambos tipos o de premium solamente
            let nombre_sala = ui_instance.get_nombre_sala(); //ambos 
            let descripcion = ui_instance.get_descripcion().to_string(); //ambos
            let num_participantes = ui_instance.get_num_participantes(); //ambos
            let mut date_inicio = ui_instance.get_date_inicio().to_string(); //premium
            let mut time_inicio = ui_instance.get_time_inicio().to_string(); //premium
            let duracion_sala = ui_instance.get_duracion_sala(); //premium
            let is_privada = ui_instance.get_is_privada(); //ambos
            let is_recurrente = ui_instance.get_is_recurrente(); //ambos
            let is_filtro_dominio = ui_instance.get_is_filtro_dominio();

            //Añadir sala al recycler
            if let Some(ui_instance) = ui_weak.upgrade() {
                let nombre_sala = ui_instance.get_nombre_sala();
                let codigo = "123456";
                
                if !nombre_sala.is_empty() && !codigo.is_empty() {
                    salas_model_add.push((nombre_sala.into(), codigo.into()));
                }
            }

            //si es premium o basico
            if let Some(usuario)=cargar_usuario(){
                //Obtenemos el id para pasarlo como parametro a crear_sala
                let mut id_usuario:i32 = 0;
                let id = recibir_id(&usuario);
                match id {
                    Ok(user_id) => id_usuario = user_id,
                    Err(error) => println!("Error obteniendo ID: {}", error),
                }
                if usuario.usuario.premium{
                    //comprobamos si los campos hora y tiempo estan vacios
                    date_inicio = establecer_fecha_predeterminada(&date_inicio);
                    time_inicio = establecer_hora_predeterminada(&time_inicio);
                    match validar_datos_sala_premium(&nombre_sala, &descripcion, num_participantes, &date_inicio, &time_inicio, duracion_sala, is_recurrente){
                        Ok(_) => {
                            //let time_cierre = hora_de_cierre(time_inicio, duracion_sala);

                            use crate::crear_sala::enviar_sala;
                            let mut envio_exitoso=false;
                            let client= client3.clone();
                            tokio::spawn(enviar_sala(client, nombre_sala.to_string(), descripcion.to_string(), num_participantes, is_privada, is_recurrente, date_inicio.to_string(), 
                                time_inicio.to_string(), duracion_sala.into(), id_usuario));
                            ui_instance.set_empty_error_sala("".into());
                            println!("sala creada");
                        }
                        Err(err) => {
                            ui_instance.set_empty_error_sala(err.into());
                            ui_instance.set_empty_error_text_color(ui_instance.get_rojo());
                            ui_instance.set_input_border_color(ui_instance.get_border());
                        }
                    }
                }
                else{
                    //usuario basico
                    match validar_datos_sala_basica(&nombre_sala, &descripcion, num_participantes){
                        Ok(_)=>{
                            println!("sala creada");
                        }
                        Err(err)=>{
                            ui_instance.set_empty_error_sala(err.into());
                            ui_instance.set_empty_error_text_color(ui_instance.get_rojo());
                            ui_instance.set_input_border_color(ui_instance.get_border());
                        }
                    }
                }
            };
        }
    });
    ui.on_subir_archivo({
        move || {
        match seleccionar_archivo(){
            Ok(Some(path)) =>{
                println!("Path del archivo: {}", path);
            }
            Ok(None) => println!("Usuario canceló"),
            Err(e) => println!("Error: {}", e),
        }
    }
    });
}

#[no_mangle]
pub extern "C" fn Java_com_example_faenaapp_MainActivity_test_1jni(
    _env: JNIEnv,
    _class: JClass,
) -> jint {
    init_logger();
    
    #[cfg(target_os = "android")]
    log::info!("🦀 Rust: Función test_jni llamada exitosamente!");
    
    42
}

#[no_mangle]
pub extern "C" fn Java_com_example_faenaapp_MainActivity_start_1rust_1ui(
    _env: JNIEnv,
    _class: JClass,
) {
    init_logger();
    
    #[cfg(target_os = "android")]
    log::info!("🦀 Rust: JNI llamado desde Java - versión simplificada");

    // Ejecutar en un hilo separado
    std::thread::spawn(|| {
        #[cfg(target_os = "android")]
        log::info!("🦀 Rust: Iniciando UI en hilo separado...");
        
        // Intentar crear la UI con configuración por defecto
        match MainWindow::new() {
            Ok(ui) => {
                #[cfg(target_os = "android")]
                log::info!("🦀 Rust: MainWindow creado exitosamente");
                
                // Mostrar la ventana
                if let Err(e) = ui.show() {
                    #[cfg(target_os = "android")]
                    log::error!("🦀 Rust: Error mostrando UI: {:?}", e);
                    return;
                }
                
                #[cfg(target_os = "android")]
                log::info!("🦀 Rust: UI mostrada, ejecutando bucle...");
                
                // Ejecutar UI
                if let Err(e) = ui.run() {
                    #[cfg(target_os = "android")]
                    log::error!("🦀 Rust: Error ejecutando UI: {:?}", e);
                }
            },
            Err(e) => {
                #[cfg(target_os = "android")]
                log::error!("🦀 Rust: Error creando MainWindow: {:?}", e);
            }
        }
        
        #[cfg(target_os = "android")]
        log::info!("🦀 Rust: Hilo de UI terminado");
    });

    #[cfg(target_os = "android")]
    log::info!("🦀 Rust: JNI completado - UI iniciada en hilo separado");
}
