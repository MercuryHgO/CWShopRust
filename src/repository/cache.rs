use dotenv::dotenv;

pub async fn create_connection() -> redis::Connection {
    dotenv().ok();
    
    let client = redis::Client::open(
        std::env::var("REDIS_URL")
            .expect("REDIS_URL env must be provided!")
    ).expect("Error opening redis client");

    client.get_connection().expect("Error connection to redis client")
}
