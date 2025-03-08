use tiberius::{AuthMethod, Client, Config};
use tokio::net::TcpStream;
use tokio_util::compat::TokioAsyncWriteCompatExt as _;

pub async fn connect_with_host_port_username_password() -> Result<(), Box<dyn std::error::Error>> {
    let mut config = Config::new();

    // Use SQL Server Authentication (user name and password)
    config.authentication(AuthMethod::sql_server("db12877", "Snakesystem@09"));

    config.host("server=db12877.public.databaseasp.net");
    config.port(22828);
    config.trust_cert();

    let tcp = TcpStream::connect(config.get_addr()).await?;
    let client = Client::connect(config, tcp.compat_write()).await?;
    println!("Connected to SQL Server");
    let _ = client.close().await?;

    Ok(())
}