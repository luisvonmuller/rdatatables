

/*
8888888b.        8888888b.        d8888 88888888888     d8888 888888b.   888      8888888888 .d8888b.
888   Y88b       888  "Y88b      d88888     888        d88888 888  "88b  888      888       d88P  Y88b
888    888       888    888     d88P888     888       d88P888 888  .88P  888      888       Y88b.
888   d88P       888    888    d88P 888     888      d88P 888 8888888K.  888      8888888    "Y888b.
8888888P"        888    888   d88P  888     888     d88P  888 888  "Y88b 888      888           "Y88b.
888 T88b  888888 888    888  d88P   888     888    d88P   888 888    888 888      888             "888
888  T88b        888  .d88P d8888888888     888   d8888888888 888   d88P 888      888       Y88b  d88P
888   T88b       8888888P" d88P     888     888  d88P     888 8888888P"  88888888 8888888888 "Y8888P"
*/

/* Form diesel and serve imports */
use diesel::*;
use rocket::request::{FormItems, FromForm};
use serde::Serialize;
use diesel::sql_types::BigInt;

/* This one stands for the r-datatables counting struct */
#[derive(QueryableByName, Serialize)]
pub struct Count {
    #[sql_type = "BigInt"]
    pub count: i64,
}

/*
    "Tables" explanation:
    ===================
    -> Data Structure comes like:
      (JoinType, (dest_table_name, dest_table_key), (origin_table_name, origin_table_key))

    -> Implemented Struct will return something like:
    "
        `JoinType` JOIN `dest_table_name`
                ON `origin_table_name`.`origin_table_key` = `table2`.`common_field` *( n-th)
    "
*/

#[derive(Debug, Clone)]
pub struct Tables<'a> {
    pub origin: (&'a str, &'a str), /* From */
    pub fields: Vec<&'a str>,       /* Fields to seek for */
    pub join_targets: Option<Vec<(&'a str, (&'a str, &'a str), (&'a str, &'a str))>>, /* Join Targets explained over here */
    pub datatables_post_query: DataTableQuery, /* Incoming Query */
    pub query: Option<String>, /* Our builded query holder */
    pub condition: Option<Vec<(&'a str, &'a str, &'a str)>>, /* (And/Or, Field_Name, Value) */
}

impl<'a> Tables<'a> {
    pub fn generate(&mut self) -> String {
        match self.datatables_post_query.order[0].0 {
            Some(column_index_to_order) => format!(
                "{} ORDER BY {} {}",
                self.select().join().where_like().condition().query.to_owned().unwrap(),
                self.fields[column_index_to_order as usize],
                &self.datatables_post_query.order[0]
                    .1
                    .as_ref()
                    .unwrap()
                    .to_uppercase()
            ),
            None => self.select().join().where_like().condition().query.to_owned().unwrap(),
        }
    }

    /* Returns fields for the query */
    pub fn select(&mut self) -> Self {
        let stmt = &self
            .fields
            .iter()
            .map(|field| format!("{}, ", field))
            .collect::<String>();

        self.query = Some(
            format!(
                "SELECT {} FROM {}",
                stmt[..(stmt.len() - 2)].to_owned(),
                self.origin.0
            )
            .to_owned(),
        );

        self.to_owned()
    }

    pub fn where_like(&mut self) -> Self {
        /* #Where like:
        ## This function receives self (as all of the SQL generators) and
        reparses the content of "where" from the incoming Datatable query
        to do a seeking for desired information over all table fields

        returns... gues what? self!
        */
        let stmt = self
            .fields
            .iter()
            .map(|field| {
                format!(
                    " CAST({} as TEXT) LIKE '%{}%' OR",
                    field,
                    self.datatables_post_query.search[0].0.as_ref().unwrap()
                )
            })
            .collect::<String>();

        self.query = Some(
            format!(
                "{} WHERE {}",
                self.query.to_owned().unwrap(),
                stmt[..(stmt.len() - 2)].to_owned()
            )
            .to_owned(),
        );

        self.to_owned()
    }

    pub fn join(&mut self) -> Self {
        /*
        # How this works?
           ## We will match the existing needing of appending the "join statement" or not
            As well we do on other self sql generators functions, we'll opt to not do an if stmt
            for seeking the "last" target and doing a exactly cut for the string to append.

            Returns self.
        */
        match self.join_targets {
            Some(_) => {
                let stmt = self
                    .join_targets
                    .as_ref()
                    .unwrap()
                    .iter()
                    .map(|(join_type, (target, target_key), (origin, origin_key))| {
                        format!(
                            "{} JOIN {} ON {}.{} = {}.{} ",
                            join_type.to_uppercase(), target, origin, origin_key, target, target_key,
                        )
                    })
                    .collect::<String>();

                self.query = Some(
                    format!("{}  {}", self.query.to_owned().unwrap(), stmt.to_owned()).to_owned(),
                );

                self.to_owned()
            }
            None => self.to_owned(),
        }
    }

    pub fn condition(&mut self) -> Self {
        match self.condition {
            Some(_) => {
                let stmt = self.condition.as_ref().unwrap().iter().map(|(sub_cond, target, value)| {
                    format!(" {} CAST({} AS TEXT) LIKE '%{}%'", sub_cond.to_uppercase(), target, &value.to_string())
                }).collect::<String>();

                self.query = Some(
                    format!("{}  {}", self.query.to_owned().unwrap(), stmt.to_owned()).to_owned(),
                );

                self.to_owned()
    
            }
            None => {
                self.to_owned()
            }
        }
    }
}


#[allow(non_snake_case)]
#[derive(Debug, Clone)]
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
        
        let tmp_columns: Vec<(
            Option<i32>,
            Option<String>,
            Option<bool>,
            Option<bool>,
            Option<String>,
            Option<bool>,
        )> = vec![(None, None, None, None, None, None)];

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

pub fn datatables_query<
    T: diesel::deserialize::QueryableByName<diesel::pg::Pg> + std::fmt::Debug + std::clone::Clone,
>(
    table: Tables,
    conn: PgConnection,
) -> OutcomeData<T> {
    let (data_results, total_data): (Vec<T>, Count) = (
        sql_query(table.clone().generate())
            .load(&conn)
            .expect("Failed to retrieve information"),
        sql_query(format!("SELECT COUNT(*) FROM {}", table.origin.0))
            .load::<Count>(&conn)
            .expect("Query failed")
            .pop()
            .expect("No rows"),
    );

    let tmp_results = data_results[(table.datatables_post_query.start as usize)..].to_vec();

    OutcomeData::<T> {
        draw: table.datatables_post_query.draw,     /* N-th draw */
        recordsTotal: total_data.count,             /* How much we have on this table */
        recordsFiltered: data_results.len() as i32, /* How much query has returned */
        data: if tmp_results.len() >= (table.datatables_post_query.length as usize) {
            tmp_results[..(table.datatables_post_query.length as usize)].to_vec()
        } else {
            tmp_results.to_vec()
        },
    }
}
