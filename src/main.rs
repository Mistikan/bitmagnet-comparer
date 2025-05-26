use chrono::{DateTime, FixedOffset};
use clap::{Args, Parser, Subcommand, crate_authors, crate_description, crate_name, crate_version};
use log::info;
use std::path::PathBuf;
use tokio_postgres::{Error, NoTls};

#[derive(Parser, Debug)]
#[command(version, author, about, long_about = None)]
#[command(
    name = crate_name!(),
    author = crate_authors!(),
    version = crate_version!(),
    about = crate_description!()
)]
struct App {
    #[clap(flatten)]
    global_opts: GlobalOpts,

    #[command(subcommand)]
    cmd: TypeInputFiles,
}

#[derive(Debug, Args)]
struct GlobalOpts {
    // Bitmagnet postgresql url
    #[clap(long, global = true)]
    bitmagnet_postgresql_url: Option<String>,
}

#[derive(Debug)]
struct File {
    path: PathBuf,
    size: i64,
    date: Option<DateTime<FixedOffset>>,
}

#[derive(Subcommand, Debug, Clone)]
enum TypeInputFiles {
    Single {
        path: PathBuf,
        size: i64,
        #[arg(value_parser = DateTime::parse_from_rfc3339)]
        date: Option<DateTime<FixedOffset>>,
    },
    FindTorrentDataPostgresql {
        postgresql_url: String,
    },
}

// Input files
async fn get_input_files(
    type_input_files: TypeInputFiles,
) -> Result<impl Iterator<Item = File>, Error> {
    let ret = match type_input_files {
        TypeInputFiles::Single { path, size, date } => {
            let input_files = vec![File { path, size, date }];
            input_files
        }
        TypeInputFiles::FindTorrentDataPostgresql { postgresql_url } => {
            info!("Connect db");
            let (client, connection) = tokio_postgres::connect(&postgresql_url, NoTls).await?;

            tokio::spawn(async move {
                if let Err(e) = connection.await {
                    eprintln!("connection error: {e}");
                }
            });

            let info_hash_iter = client
                .query(
                    "
                UPDATE file_info x
                SET bitmagnet_created_at = Now()
                FROM file_info y
                WHERE x.id = y.id
                RETURNING y.path, y.size, y.bitmagnet_created_at;
                ",
                    &[],
                )
                .await?
                .into_iter()
                .map(|row| {
                    let path: String = row.get(0);
                    let path = PathBuf::from(path);
                    let size = row.get(1);
                    let created_at = row.get(2);
                    File {
                        path,
                        size,
                        date: created_at,
                    }
                });
            info_hash_iter.collect()
        }
    };

    Ok(ret.into_iter())
}

async fn search_by_file(
    file: &File,
    postgresql_url: &str,
) -> Result<impl Iterator<Item = Vec<u8>> + use<>, Error> {
    info!("Connect db");
    let (client, connection) = tokio_postgres::connect(postgresql_url, NoTls).await?;

    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("connection error: {e}");
        }
    });

    let file_extension = file.path.extension().unwrap().to_str();
    info!("Extension: {:?}, Size: {}", file_extension, file.size);

    let info_hash_iter = client
        .query(
            "
        SELECT info_hash
        FROM torrent_files
        WHERE size = $1 AND extension = $2 AND ($3::timestamptz IS NULL OR created_at > $3)
        UNION
        SELECT info_hash
        FROM torrents
        WHERE size = $1 AND extension = $2 AND ($3::timestamptz IS NULL OR created_at > $3)
        ;
        ",
            &[&file.size, &file_extension, &file.date],
        )
        .await?
        .into_iter()
        .map(|row| {
            let info_hash: Vec<u8> = row.get(0);
            info_hash
        });

    Ok(info_hash_iter)
}

#[tokio::main]
async fn main() {
    json_log::init_from_env().unwrap();

    let cli = App::parse();

    info!("Get input files");
    let type_input_files = cli.cmd;
    // TODO: когда я ищу файл, я должен ставить метку, что я уже искал ранее по дате.
    // Это же и дублировать в search hashs, ищя по базе данных с датой
    let input_files = get_input_files(type_input_files).await.unwrap();

    let bitmagnet_postgresql_url = cli
        .global_opts
        .bitmagnet_postgresql_url
        .expect("Set bitmagnet postgresql url");

    info!("Search hashs");
    let info_hashs = input_files.map(async |input_file| {
        info!("Get hash for file: {input_file:?}");

        search_by_file(&input_file, &bitmagnet_postgresql_url)
            .await
            .unwrap()
    });

    info!("Print hashs");
    let info_hashs = info_hashs.map(async |info_hash| {
        let info_hash_vec = info_hash.await;

        info_hash_vec.map(hex::encode)
    });

    for info_hash in info_hashs {
        let info_hash = info_hash.await;
        for hex_string in info_hash {
            println!("{hex_string}");
        }
    }
}
