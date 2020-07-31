/* Form */
use rocket::request::LenientForm;
use rocket::request::{FormItems, FromForm};
use serde::Serialize;
use std::collections::HashMap;
use diesel::*;

#[allow(non_snake_case)]
#[derive(Debug)]
pub struct DataTableQuery {
    pub draw: i32, /* Stands for the n-th time that we're drawing */
    pub columns: Vec<(
        Option<i32>,
        Option<String>,
        Option<bool>,
        Option<bool>,
        Option<String>,
        Option<bool>,
    )>,
    pub order: Vec<(Option<i32>, Option<String>)>,
    pub start: i32,  /* How much to skip */
    pub length: i32, /* How much to retrieve */
    pub search: Vec<(Option<String>, bool)>,
    pub info: Option<i32>,
}

impl<'f> FromForm<'f> for DataTableQuery {
    // In practice, we'd use a more descriptive error type.
    type Error = ();

    fn from_form(items: &mut FormItems<'f>, strict: bool) -> Result<DataTableQuery, ()> {
        let mut draw: Option<i32> = None;
        let mut start: Option<i32> = None;
        let mut length: Option<i32> = None;

        /* Tmp columns holding vector */
        let mut tmp_columns: Vec<(
            Option<i32>,
            Option<String>,
            Option<bool>,
            Option<bool>,
            Option<String>,
            Option<bool>,
        )> = vec![(None, None, None, None, None, None)];

        let mut column_tuple: (
            Option<i32>,
            Option<String>,
            Option<bool>,
            Option<bool>,
            Option<String>,
            Option<bool>,
        ) = (None, None, None, None, None, None);

        let mut order_tuple: (Option<i32>, Option<String>) = (None, None);

        let mut search_value: Option<String> = None;

        let mut time_stamp: Option<i32> = None;

        for item in items {
            match item.key.as_str() {
                "draw" if draw.is_none() => {
                    let decoded = item.value.url_decode().map_err(|_| ())?;
                    draw = Some(match decoded.parse::<i32>() {
                        Ok(item_val) => item_val,
                        Err(_err_msg) => 0,
                    });
                }
                "start" if start.is_none() => {
                    let decoded = item.value.url_decode().map_err(|_| ())?;
                    start = Some(match decoded.parse::<i32>() {
                        Ok(item_val) => item_val,
                        Err(_err_msg) => 0,
                    });
                }
                "length" if length.is_none() => {
                    let decoded = item.value.url_decode().map_err(|_| ())?;
                    length = Some(match decoded.parse::<i32>() {
                        Ok(item_val) => item_val,
                        Err(_err_msg) => 0,
                    });
                }
                "search%5Bvalue%5D" if search_value.is_none() => {
                    let decoded = Some(item.value.url_decode().map_err(|_| ())?);
                    search_value = decoded;
                }
                key if key.contains("columns") => {
                    let key = item.key.url_decode().unwrap();

                    /* The first indexing dude */
                    let (init, end) = (
                        *&key.find("[").unwrap() as usize,
                        *&key.find("]").unwrap() as usize,
                    );

                    /* Since the first returns [0.. we will need to add an postion to our array */
                    let vector_index = &key.as_str()[(init + 1)..end].parse::<i32>().unwrap();

                    for array_space in key.split(&format!("columns[{}]", vector_index)) {
                        /* This must be refactored to a non-exaustive match statement */
                        if array_space.contains("data") {
                            let decoded = item.value.url_decode().map_err(|_| ())?;
                            column_tuple.0 = Some(match decoded.parse::<i32>() {
                                Ok(item_val) => item_val,
                                Err(_err_msg) => 0,
                            });
                        }

                        if array_space.contains("name") {
                            column_tuple.1 = Some(item.value.url_decode().map_err(|_| ())?);
                        }

                        if array_space.contains("searchable") {
                            column_tuple.2 = Some(
                                item.value
                                    .url_decode()
                                    .map_err(|_| ())?
                                    .parse::<bool>()
                                    .unwrap(),
                            );
                        }
                        if array_space.contains("orderable") {
                            column_tuple.3 = Some(
                                item.value
                                    .url_decode()
                                    .map_err(|_| ())?
                                    .parse::<bool>()
                                    .unwrap(),
                            );
                        }
                        
                        if array_space.contains("search][value]") {
                            column_tuple.4 = Some(item.value.url_decode().map_err(|_| ())?);
                        }
                        
                        if array_space.contains("search][regex]") {
                            column_tuple.5 = Some(
                                item.value
                                    .url_decode()
                                    .map_err(|_| ())?
                                    .parse::<bool>()
                                    .unwrap(),
                            );
                            /* Since we have parsed all fields of this tupple, we can now appent */
                            tmp_columns.push(column_tuple);
                            /* And also clean it */
                            column_tuple = (None, None, None, None, None, None);
                        }
                    }
                    /* Array index and now I'll keep going to search to next position values */
                }
                key if key.contains("order%5B0%5D") => {
                    if key.contains("order%5B0%5D%5Bcolumn%5D") {
                        order_tuple.0 = Some(
                            item.value
                                .url_decode()
                                .map_err(|_| ())?
                                .parse::<i32>()
                                .unwrap(),
                        );
                    } else {
                        order_tuple.1 = Some(item.value.url_decode().map_err(|_| ())?);
                    }
                }
                "_" => {
                    time_stamp = Some(
                        item.value
                            .url_decode()
                            .map_err(|_| ())?
                            .parse::<i32>()
                            .unwrap(),
                    );
                }
                _ if strict => return Err(()),
                _ => {}
            }
        }

        Ok(DataTableQuery {
            draw: match draw {
                Some(value) => value,
                None => 0,
            },
            columns: tmp_columns[1..].to_owned(),
            order: vec![order_tuple],
            start: match start {
                Some(value) => value,
                None => 0,
            },
            length: match length {
                Some(value) => value,
                None => 0,
            },
            search: vec![(search_value, false)],
            info: time_stamp.to_owned(),
        })
    }
}

#[allow(non_snake_case)]
#[derive(Debug, Serialize)]
pub struct OutcomeData<T> {
    pub draw: i32,
    pub recordsTotal: i64,
    pub recordsFiltered: i32,
    pub data: Vec<T>,
}

use diesel::sql_types::BigInt;
#[derive(QueryableByName)]
pub struct Count {
    #[sql_type = "BigInt"]
    pub count: i64,
}

pub fn datatables_query<
    T: diesel::deserialize::QueryableByName<diesel::pg::Pg> + std::fmt::Debug + std::clone::Clone,
>(
    table: &str,
    columns: HashMap<i32, &str>,
    incoming_query: LenientForm<DataTableQuery>,
    conn: PgConnection,
) -> OutcomeData<T> {

    let mut fields: String = String::from("");

    for (_pos, field) in columns.clone() {
        if fields.len() > 1 {
            fields = format!("{}, {}", fields, field);
        } else {
            fields = field.to_string();
        }
    }

    let mut fields_like: String = String::from("");

    for (_pos, field) in columns.clone() {
        if fields_like.len() > 1 {
            fields_like = format!(
                "{} OR CAST({} AS TEXT) LIKE '%{}%'",
                fields_like,
                field,
                incoming_query.search[0].clone().0.unwrap()
            );
        } else {
            fields_like = format!(
                "WHERE CAST({} as TEXT) LIKE '%{}%'",
                field,
                incoming_query.search[0].clone().0.unwrap()
            );
        }
    }

    let query: String = match &incoming_query.order[0].0 {
        Some(column_index_to_order) => format!(
            "SELECT {} FROM {} {} ORDER BY {} {}",
            fields.clone(),
            table.clone(),
            fields_like.clone(),
            columns[column_index_to_order],
            &incoming_query.order[0].1.as_ref().unwrap().to_uppercase()
        )
        .to_owned(),
        None => format!("SELECT {} FROM {}", fields, table,).to_owned(),
    };

    let (data_results, total_data): (Vec<T>, Count) = (
        sql_query(query.clone())
            .load(&conn)
            .expect("Failed to retrieve information"),
        sql_query(format!("SELECT COUNT(*) FROM {}", table))
            .load::<Count>(&conn)
            .expect("Query failed")
            .pop()
            .expect("No rows"),
    );

    let tmp_results = data_results[(incoming_query.start as usize)..].to_vec();

    OutcomeData::<T> {
        draw: incoming_query.draw,                  /* N-th draw */
        recordsTotal: total_data.count,             /* How much we have on this table */
        recordsFiltered: data_results.len() as i32, /* How much query has returned */
        data: if tmp_results.len() >= (incoming_query.length as usize)  {
            tmp_results[..(incoming_query.length as usize)].to_vec()
        } else {
            tmp_results.to_vec()
        },
    }
}
