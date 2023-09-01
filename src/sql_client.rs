use anyhow::Ok;
use async_std::net::TcpStream;
use tiberius::{Client, Config, Query, QueryItem};
use tiberius::SqlBrowser;
use once_cell::sync::Lazy;
use std::env;
use futures_util::stream::TryStreamExt;
use chrono::NaiveDate;

static CONN_STR: Lazy<String> = Lazy::new(|| {
    env::var("TIBERIUS_TEST_CONNECTION_STRING").unwrap_or_else(|_| {
        "server=tcp:127.0.0.1\\sql2022d;IntegratedSecurity=true;TrustServerCertificate=true".to_owned()
    })
});

static JDBC_CONN_STR_NAMED_INSTANCE: Lazy<String> = Lazy::new(|| {
    env::var("TIBERIUS_TEST_JDBC_CONNECTION_STRING").unwrap_or_else(|_| {
        "jdbc:sqlserver://127.0.0.1\\sql2022d;integratedSecurity=true;trustServerCertificate=true".to_owned()
    })
});

static JDBC_CONN_STR_PORT: Lazy<String> = Lazy::new(|| {
    env::var("TIBERIUS_TEST_CONNECTION_STRING_WITH_PORT").unwrap_or_else(|_| {
        "jdbc:sqlserver://127.0.0.1:22828;database=DestinationDB;user=developer;password=developer;trustServerCertificate=true".to_owned()
    })
});

/// Connect to a named instance of SQL Server through SQL Server Browser.
/// Make sure that SQL Server Browser is installed and running, otherwise
/// the connection will fail.
pub async fn connect_through_sql_browser() -> anyhow::Result<()> {
    let mut config = Config::new();

    // Use Windows Authentication
    config.authentication(tiberius::AuthMethod::Integrated);

    config.host("127.0.0.1");

    // The default port of SQL Browser
    config.port(1434);

    // The name of the database server instance.
    config.instance_name("sql2022d");

    // it is not a good idea to do this in production
    config.trust_cert();

    // This will create a new `TcpStream` from `async-std`, connected to the
    // right port of the named instance.
    let tcp = TcpStream::connect_named(&config).await?;

    // Perform the real connection to the SQL Server
    let client = Client::connect(config, tcp).await?;
    println!("Successfully connected to server.");

    // And then close the connection.
    client.close().await?;
    Ok(())
}


/// Connect to a named instance of SQL Server without specifing the port number.
/// SQL Server Browser must be running and will automatically choose the right port.
async fn connect_to_named_instance() -> anyhow::Result<()> {
    let config = Config::from_ado_string(&CONN_STR)?;

    let tcp = TcpStream::connect_named(&config).await?;
    tcp.set_nodelay(true)?;

    let client = Client::connect(config, tcp).await?;
    println!("Successfully connected to server.");

    // And then close the connection.
    client.close().await?;

    Ok(())
}

// Connect to a named SQL Server instance using a JDBC connection string
async fn connect_with_jdbc_connection_string() -> anyhow::Result<()> {
    let config = Config::from_jdbc_string(&JDBC_CONN_STR_NAMED_INSTANCE)?;

    let tcp = TcpStream::connect_named(&config).await?;
    tcp.set_nodelay(true)?;

    let client = Client::connect(config, tcp).await?;
    println!("Successfully connected to server.");

    // And then close the connection.
    client.close().await?;

    Ok(())
}

// Connect to a named SQL Server instance using a JDBC connection string
async fn connect_with_jdbc_connection_string_port() -> anyhow::Result<()> {
    let config = Config::from_jdbc_string(&JDBC_CONN_STR_PORT)?;

    let tcp = TcpStream::connect_named(&config).await?;
    tcp.set_nodelay(true)?;

    let client = Client::connect(config, tcp).await?;
    println!("Successfully connected to server.");

    // And then close the connection.
    client.close().await?;

    Ok(())
}


async fn create_table()-> anyhow::Result<()> {
    let config = Config::from_ado_string(&CONN_STR)?;

    let tcp = TcpStream::connect(config.get_addr()).await?;

    tcp.set_nodelay(true)?;

    let mut client = Client::connect(config, tcp).await?;

    let select = Query::new("
    drop table if exists dbo.rabbit_births
    create table dbo.rabbit_births
    (
        id int,
        name varchar(max),
        date_of_birth datetime
    )
    ");
    
    let result=select.execute(&mut client).await?;

    // Print the total number of rows affected
    println!("Rows affected: {}",result.total());
    client.close().await?;

    Ok(())
    
}

async fn insert_data()->anyhow::Result<()> {
    let config = Config::from_ado_string(&CONN_STR)?;

    let tcp = TcpStream::connect(config.get_addr()).await?;

    tcp.set_nodelay(true)?;

    let mut client = Client::connect(config, tcp).await?;

    let result = client.execute(
        "INSERT INTO rabbit_births (id, name, date_of_birth) VALUES (@P1, @P2, @P3)", 
        &[&1i32, &"Bugs Bunny", &"2023-08-01"],
    ).await?;
    
    
    println!("Rows affected: {}",result.total());
    client.close().await?;

    Ok(())
}


async fn read_table() -> anyhow::Result<()> {

    let config = Config::from_ado_string(&CONN_STR)?;
    let tcp = TcpStream::connect(config.get_addr()).await?;
    tcp.set_nodelay(true)?;

    let mut client = Client::connect(config, tcp).await?;
    let select = Query::new("SELECT *from rabbit_births");
    let mut stream=select.query(&mut client).await?;

    // Read each row as long as ther arrive from the stream
    while let Some(row)=stream.try_next().await? {
        
        match row {

            // Metadata of the result set
            QueryItem::Metadata(meta) => {
                println!("{:?}",meta);
            },

            // Actual data rows
            QueryItem::Row(r) => {
                
                // Break line to separate each row
                println!();

                // Create variables with an explicit type annotation.
                let date_column: Option<chrono::NaiveDate> = None;//r.get(2);
                let name_column: Option<&str> =r.get(1);
                let id_column: Option<i32> =r.get(0);
                
                // The complete list of SQL Server data types
                // matching with Rust data types can be found in:
                // https://docs.rs/tiberius/latest/tiberius/trait.FromSql.html#tymethod.from_sql


                // Print an INT column
                if let Some(value) = id_column {
                    println!("{:?}", value);
                }else{
                    println!("NULL");
                }
                
                // Print an VARCHAR column
                if let Some(value) =name_column {
                    println!("{:?}", value);
                }else{
                    println!("NULL");
                }
                
                // Print a DATE column
                if let Some(value) = date_column {
                    println!("{:?}", value);
                }else{
                    println!("NULL");
                }
            },
        }
    }

    Ok(())
}


async fn execute_stored_procedure()->anyhow::Result<()> {
    let config = Config::from_ado_string(&CONN_STR)?;
    let tcp = TcpStream::connect(config.get_addr()).await?;
    tcp.set_nodelay(true)?;

    let mut client = Client::connect(config, tcp).await?;
 
    let result = client.execute(
        "exec dbo.register_rabbit_birth @birth_date= @P1, @name=@P2", 
        &[&"2023-08-24", &"Lola Bunny"],
    ).await?;
    
    
    println!("Rows affected: {}",result.total());
    client.close().await?;

    Ok(())
}


async fn create_stored_procedure()->anyhow::Result<()> {
    let config = Config::from_ado_string(&CONN_STR)?;
    let tcp = TcpStream::connect(config.get_addr()).await?;
    tcp.set_nodelay(true)?;

    let mut client = Client::connect(config, tcp).await?;
    
    let _=client.simple_query("CREATE or alter procedure register_rabbit_birth\r
    @birth_date date,\r
    @name varchar(max)\r
\r
as\r
\r
    declare @new_id int\r
    select @new_id=max(id)+1from rabbit_births\r
      
    insert dbo.rabbit_births(id,name,date_of_birth)\r
    values(@new_id,@name,@birth_date)\r
    ").await?;
   
    println!("Stored procedure created or altered");

    Ok(())

}









async fn execute_stored_procedure_with_output_parameter()->anyhow::Result<()> {
    let config = Config::from_ado_string(&CONN_STR)?;
    let tcp = TcpStream::connect(config.get_addr()).await?;
    tcp.set_nodelay(true)?;

    let mut client = Client::connect(config, tcp).await?;

    //let results=client.execute("dbo.register_rabbit_birth_and_get_id @birth_date= @P1, @name=@P2, @id=@P3 OUTPUT",
    //    &[&"2023-08-24", &"Clyde Bunny", &0i32]).await?;

    let mut results=client.query("
    declare @id int
exec dbo.register_rabbit_birth_and_get_id @birth_date= @P1, @name=@P2, @id=@id OUTPUT
select @id
    ",
    &[&"2023-08-24", &"Clyde Bunny"]).await?;

    while let Some(row)=results.try_next().await? {
        if let QueryItem::Row(r)=row{
            let new_id:Option<i32>=r.get(0);
            if let Some(value)=new_id{
                println!("New id={:?}",value);
            }
        }
    }
    
    Ok(())

}


async fn execute_stored_procedure_with_return_value()->anyhow::Result<()> {
    let config = Config::from_ado_string(&CONN_STR)?;
    let tcp = TcpStream::connect(config.get_addr()).await?;
    tcp.set_nodelay(true)?;

    let mut client = Client::connect(config, tcp).await?;

    let mut select = Query::new("
    declare @return_value int
    exec @return_value= dbo.register_rabbit_birth_return_id @birth_date= @P1, @name=@P2
    select @return_value");
    select.bind("2023-08-24");
    select.bind("Clyde Bunny");
    
    let mut stream=select.query(&mut client).await?;

    while let Some(row)=stream.try_next().await? {
        if let QueryItem::Row(r)=row{
            let return_value:Option<i32>=r.get(0);
            if let Some(value)=return_value{
                println!("Return value: {:?}",value);
            }
        }
        else{
            println!("Nothing as return value: {:?}",row);
        }
    }
    
    Ok(())
}


async fn create_scalar_function()->anyhow::Result<()> {
    let config = Config::from_ado_string(&CONN_STR)?;
    let tcp = TcpStream::connect(config.get_addr()).await?;
    tcp.set_nodelay(true)?;

    let mut client = Client::connect(config, tcp).await?;
    let mut result=client.simple_query("
create or alter function dbo.reverse_words\r\n
(\r
    @original_value varchar(100)\r
)\r
/* Reverse every word in a phrase. */\r
returns varchar(100)\r
as\r
begin\r
    
    declare @res table\r
    (\r
        ordinal bigint,\r
        value varchar(100)\r
    )\r
\r
    insert @res(ordinal,value)\r
    select ordinal,value\r
    from string_split(@original_value,' ',1)\r
\r        
    declare @i int=1,\r
            @total int,\r
            @chunk varchar(100)\r
\r
    select @total=count(1)from @res\r
\r
    declare @new_value varchar(100)\r
    while @i<=@total\r
    begin\r
        select @chunk=value from @res\r
        where\r
        ordinal=@i\r

        if @new_value is null\r
            set @new_value=''\r

        set @new_value+=concat(reverse(@chunk),' ')\r
        
        set @i+=1\r
    end\r
\r 
    return @new_value\r
end").await?;

    println!("Function created or altered");

    // No results
    while let Some(row)=result.try_next().await? {

        println!("Row: {:?}",row);
        if let QueryItem::Row(value)=row{
            println!("{:?}",value);
        }
    }

    Ok(())
}
