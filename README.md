# rdatatables
This is the "backend" implementation for the datatables AJAX api on rust-lang.

# Follow this on how to use: 
* Add rdatatables = "0.0.1" on your crate;
* use rdatatables::*; (You'll need all of it anyways);
* First you will need a listener function for your "ajax"-POST. 
e.g:

```rust
/* Your struct */
use crate::models::DataTablesSysUserListing;

#[post("/list", data = "<query>")]
pub fn list(
    _adminitrative: AdminUser,
    query: LenientForm<DataTableQuery>,
) -> Json<OutcomeData<DataTablesSysUserListing>> {
    /* Front-end to back-end column mapping */
    let mut collumn_mapping = HashMap::new();

    /* This hashmap matches the comming struct on the front-end, columns object array on js. */
    collumn_mapping.insert(0, "user_name");
    collumn_mapping.insert(1, "user_balance");
    collumn_mapping.insert(2, "user_creation");
    collumn_mapping.insert(3, "user_lasttimeonline");
    collumn_mapping.insert(4, "user_status");
    collumn_mapping.insert(5, "user_id");

    Json(datatables_query::<DataTablesSysUserListing>(
        "sysuser",
        collumn_mapping,
        query,
        crate::establish_connection()
    ))
}
```

* Then your struct (that must be QuerybleByName and also Identifiable), like this
```rust 
#[derive(Debug, QueryableByName, Serialize, Clone)]
#[table_name = "sysuser"]
pub struct DataTablesSysUserListing {
    pub user_name: String,
    pub user_balance: f64,
    pub user_creation: NaiveDateTime,
    pub user_lasttimeonline: Option<NaiveDateTime>,
    pub user_status: Option<bool>,
    pub user_id: i32,
}

```
* Your front-end association with datatable must be something like this, do not forget about the "post" and "columns",
```js
$('#clientsTable').DataTable({
			"serverSide": true,
			"ajax": {
				url: '/admin/clients/list',
				type: 'POST',
			},
			"columns": [
				{ "data": "user_name" },
				{ "data": "user_balance", "render": (data, type, row) => {
					return data.toFixed(2).replace('.', ',')
				} },
				{
					"data": "user_creation", "render": (data, type, row) => {
						return parseData(data)
					}
				},
				{
					"data": "user_lasttimeonline", "render": (data, type, row) => {
						return parseData(data)
					}
				},
				{
					"data": "user_status", "render": (data, type, row) => {
						return statusBtn(1, data)
					}
				},
				{
					"data": "user_id", "render": () => {
						return optionsBtn(1)
					}
				}
			],
			"ordering": true,
			"info": true,
			"processing": true
});

```

Have fun. 
follow me on twitter: @luisvonmuller 

Whats missing? 
* Regex searching
* combinative column searching
* Fixing performance issues over the iterator of column parser
* Other small things like getting rid of rocket full crate as a dependence. (We're using it now for Forms implementations.)
