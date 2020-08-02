# rdatatables
This is the "backend" implementation for the datatables AJAX api on rust-lang.

# Follow this on how to use: 
* Add rdatatables = "0.0.1" on your crate;
* use rdatatables::*; (You'll need all of it anyways);
* First you will need a listener function for your "ajax"-POST. 
e.g:

```rust
/* Your datat structers */
use crate::models::rdatatables::{
    ClerksViewListing, ClerksViewListingClerkInfo, ClerksViewListingStatusClerk,
};

#[post("/list", data = "<query>")]
pub fn list(
    _adminitrative: AdminUser,
    query: LenientForm<DataTableQuery>,
) -> Json<
    OutcomeData<(
        ClerksViewListing,
        ClerksViewListingStatusClerk,
        ClerksViewListingClerkInfo,
    )>,
> {
    Json(datatables_query::<(
        ClerksViewListing,
        ClerksViewListingStatusClerk,
        ClerksViewListingClerkInfo,
    )>(
        Tables {
            origin: ("sysuser", "user_id"), /* From */
            fields: vec![
                "clerk_info.clerk_image",
                "sysuser.user_name",
                "sysuser.user_uni",
                "sysuser.user_balance",
                "status_clerk.is_available",
                "sysuser.user_status",
                "sysuser.user_id",
            ], /* Fields to seek for */
            join_targets: Some(vec![ /* If you desire to not do a Join at all, just give a None here */
                /* (join_type, (target, target_key), (origin, origin_key) */
                ("inner", ("clerk_info", "user_id"), ("sysuser", "user_id")),
                (
                    "inner",
                    ("status_clerk", "clerk_id"),
                    ("sysuser", "user_id"),
                ),
            ]),
            datatables_post_query: query.into_inner(), /* Incoming Query parses to the desired struct. */
            query: None,                               /* Our builded query holder */
        },
        crate::establish_connection(),
    ))
}
```

* Then your struct (that must be QuerybleByName and also Identifiable), like this
```rust 

/* ClerksViewListing (Implements joins) */
#[derive(Debug, QueryableByName, Serialize, Clone)]
#[table_name = "sysuser"]
pub struct ClerksViewListing {
    pub user_name: String,
    pub user_uni: Option<String>,
    pub user_balance: f64,
    pub user_id: i32,
}

#[derive(Debug, QueryableByName, Serialize, Clone)]
#[table_name = "status_clerk"]
pub struct ClerksViewListingStatusClerk {
    pub is_available: Option<i32>,
}


#[derive(Debug, QueryableByName, Serialize, Clone)]
#[table_name = "clerk_info"]
pub struct ClerksViewListingClerkInfo {
    pub clerk_image: Option<String>,
}


```
* Your front-end association with datatable must be something like this, do not forget about the "post" and "columns",
```js
$('#clientsTable').DataTable({
			"serverSide": true,
			"ordering": true,
			"info": true,
			"ajax":
			{
				url: "/admin/clerk/list",
				type: "POST"
			},
			"columns": [
				{ "data": "1.clerk_image" },
				{ "data": "0.user_name" }, 
				{ "data": "0.user_uni" },
				{ "data": "0.user_balance" },
				{ "data": "2.is_available" },
				{ "data": "0.user_status" },
				{ "data": "0.user_id" }, /* Stands for user_id */
			]
});

```

Have fun. 
follow me on twitter: @luisvonmuller 
## I'm open to oportunities ;) luis@vonmuller.com.br

Whats missing? 
* Regex searching
* combinative column searching
* Fixing performance issues over the iterator of column parser (By now the lib is parsing columns by an exaustive sequencial IF statement. Must be fixed.)
