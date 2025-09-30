use std::io::BufReader;
use calamine::Table;

use crate::mostrar_error;



/*pub fn leer_excel(bytes:Vec<u8>){
    let lector = std::io::Cursor::new(&bytes);
    match calamine::open_workbook_from_rs(lector){
        Ok(mut workbook) =>{

        }
        Err(err)=>{
        }
    }
}
    */