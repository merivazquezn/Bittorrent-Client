use super::errors::DownloadManagerError;
use super::types::Piece;
use crate::logger::CustomLogger;
use crate::server::client_has_piece;
use log::*;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::copy;
use std::io::Write;
use std::path::Path;

const LOGGER: CustomLogger = CustomLogger::init("Piece Saver");
/// Creates a directory if it doesn't exist.
/// Receives the path of the directory
/// If it already exists, does nothing.
///
pub fn create_directory(path: &str) -> Result<(), DownloadManagerError> {
    if !path.is_empty() && !std::path::Path::new(path).exists() {
        std::fs::create_dir_all(path)
            .map_err(|_| DownloadManagerError::CreateDirectoryError(path.to_string()))?;
    }

    Ok(())
}

/// Saves in disk a non-empty piece in the specified path with the number of piece as file name
/// If the piece is empty, it returns an error
///
/// Returns a Result
///
/// ## On Success
///  Returns Ok(())
///
/// ## On Error
///  A [`DownloadManagerError`]
///
/// ## Example
///
/// ```
/// use bittorrent_rustico::download_manager::{Piece, save_piece_in_disk};
///
/// let mut piece = Piece {
///     piece_number: 1,
///     data: vec![1, 2, 3],
/// };
///
/// save_piece_in_disk(&mut piece, "./src/download_manager/test_downloads").unwrap();
///
/// assert!(std::path::Path::new("./src/download_manager/test_downloads").exists());
///
/// ```
pub fn save_piece_in_disk(
    piece: &Piece,
    downloads_dir_path: &str,
) -> Result<(), DownloadManagerError> {
    if piece.data.is_empty() {
        return Err(DownloadManagerError::EmptyPieceError);
    }
    create_directory(downloads_dir_path)?;

    let mut file = File::create(format!("{}/{}", downloads_dir_path, piece.piece_number))?;
    file.write_all(&piece.data[..])?;
    Ok(())
}

pub fn join_all_pieces(
    piece_count: u32,
    target_file_name: &str,
    downloads_dir_path: &str,
) -> Result<(), DownloadManagerError> {
    File::create(format!("{}/{}", downloads_dir_path, target_file_name))?;
    let mut target_file: File = OpenOptions::new()
        .write(true)
        .append(true)
        .open(format!("{}/{}", downloads_dir_path, target_file_name))?;

    LOGGER.info(format!("Joining pieces to {}", target_file_name));
    for piece_no in 0..piece_count {
        info!(
            "joining pieces of {}/pieces/{}",
            downloads_dir_path, piece_no
        );
        let mut piece_file: File = OpenOptions::new()
            .read(true)
            .open(format!("{}/pieces/{}", downloads_dir_path, piece_no))
            .map_err(|_| DownloadManagerError::MissingPieceError(piece_no))?;

        copy(&mut piece_file, &mut target_file)?;
    }

    Ok(())
}

pub fn delete_pieces_files(pieces_dir: &str) -> Result<(), DownloadManagerError> {
    let path: &Path = Path::new(pieces_dir);
    std::fs::remove_dir_all(path)?;
    Ok(())
}

pub fn make_target_file(
    piece_count: u32,
    target_file_name: &str,
    downloads_dir_path: &str,
    persist_pieces: bool,
) -> Result<(), DownloadManagerError> {
    join_all_pieces(piece_count, target_file_name, downloads_dir_path)?;
    info!("Pieces were joined");
    if !persist_pieces {
        delete_pieces_files(format!("{}/pieces", downloads_dir_path).as_str())?;
    }

    Ok(())
}

pub fn get_existing_pieces(piece_count: u32, pieces_dir: &str) -> Vec<u32> {
    let mut pieces: Vec<u32> = Vec::new();
    for i in 0..piece_count {
        if client_has_piece(i as usize, pieces_dir) {
            pieces.push(i);
        }
    }

    pieces
}

mod tests {
    #[allow(unused_imports)]
    use super::*;
    #[allow(unused_imports)]
    use std::io::Read;

    #[test]
    fn saves_little_piece_in_disk_content_is_correct() {
        let piece_number = 1;
        let mut piece = Piece {
            piece_number,
            data: Vec::new(),
        };

        piece.data.push(1);

        let downloads_dir_path = "./src/download_manager/test_downloads";
        let result = save_piece_in_disk(&piece, downloads_dir_path);
        match result {
            Ok(_) => assert!(true),
            Err(err) => panic!("Error saving piece in disk: {}", err),
        };

        let mut file: File =
            File::open(format!("{}/{}", downloads_dir_path, piece_number)).unwrap();
        let mut buf: Vec<u8> = Vec::new();
        let bytes_read = file.read_to_end(&mut buf).unwrap();

        assert_eq!(bytes_read, 1);
        assert_eq!(buf[0], 1);
    }

    #[test]
    fn saves_big_piece_in_disk_content_is_correct() {
        let piece_number = 14;
        let piece_length: u64 = 10000;
        let mut piece = Piece {
            piece_number,
            data: Vec::new(),
        };

        for i in 0..piece_length {
            piece.data.push(i as u8);
        }

        let downloads_dir_path = "./src/download_manager/test_downloads";
        let result = save_piece_in_disk(&piece, downloads_dir_path);
        match result {
            Ok(_) => assert!(true),
            Err(err) => panic!("Error saving piece in disk: {}", err),
        };

        let mut file: File =
            File::open(format!("{}/{}", downloads_dir_path, piece_number)).unwrap();
        let mut buf: Vec<u8> = Vec::new();
        let bytes_read = file.read_to_end(&mut buf).unwrap();

        assert_eq!(bytes_read, 10000);
        assert_eq!(buf, piece.data);
    }

    #[test]
    fn fails_when_trying_to_save_piece_with_no_data() {
        let piece_number = 3;
        let piece = Piece {
            piece_number,
            data: Vec::new(),
        };

        let downloads_dir_path = "./src/download_manager/test_downloads";
        let result = save_piece_in_disk(&piece, downloads_dir_path);
        match result {
            Ok(_) => panic!("Should have failed when trying to save piece with no data"),
            Err(err) => assert!(matches!(err, DownloadManagerError::EmptyPieceError)),
        };
    }

    #[test]
    fn joins_all_3_pieces_all_exist_returns_ok() {
        fn join_vec(a: Vec<u8>, mut b: Vec<u8>) -> Vec<u8> {
            let mut c = a;
            c.append(&mut b);
            c
        }

        let piece_count = 3;

        let mut file_0 = File::create(format!(
            "./src/download_manager/test_downloads/join/test_1/pieces/0",
        ))
        .unwrap();
        let mut buf_0: Vec<u8> = Vec::new();
        for i in 0..100 {
            buf_0.push(i as u8);
        }
        file_0.write_all(buf_0.as_slice()).unwrap();

        let mut file_1 = File::create(format!(
            "./src/download_manager/test_downloads/join/test_1/pieces/1",
        ))
        .unwrap();
        let mut buf_1: Vec<u8> = Vec::new();
        for i in 0..100 {
            buf_1.push(i as u8);
        }
        file_1.write_all(buf_1.as_slice()).unwrap();

        let mut file_2 = File::create(format!(
            "./src/download_manager/test_downloads/join/test_1/pieces/2",
        ))
        .unwrap();
        let mut buf_2: Vec<u8> = Vec::new();
        for i in 0..100 {
            buf_2.push(i as u8);
        }
        file_2.write_all(buf_2.as_slice()).unwrap();

        join_all_pieces(
            piece_count,
            "target",
            "./src/download_manager/test_downloads/join/test_1",
        )
        .unwrap();

        let mut target_file = File::open(format!(
            "./src/download_manager/test_downloads/join/test_1/target"
        ))
        .unwrap();

        let mut res_buf: Vec<u8> = Vec::new();
        target_file.read_to_end(&mut res_buf).unwrap();

        assert_eq!(res_buf.len(), 300);

        let expected_buf: Vec<u8> = join_vec(join_vec(buf_0, buf_1), buf_2);

        assert_eq!(res_buf, expected_buf);
    }

    #[test]
    fn joins_all_3_pieces_final_file_missing_returns_error() {
        let piece_count = 3;

        let mut file_0 = File::create(format!(
            "./src/download_manager/test_downloads/join/test_2/pieces/0",
        ))
        .unwrap();
        let mut buf_0: Vec<u8> = Vec::new();
        for i in 0..100 {
            buf_0.push(i as u8);
        }
        file_0.write_all(buf_0.as_slice()).unwrap();

        let mut file_1 = File::create(format!(
            "./src/download_manager/test_downloads/join/test_2/pieces/1",
        ))
        .unwrap();
        let mut buf_1: Vec<u8> = Vec::new();
        for i in 0..100 {
            buf_1.push(i as u8);
        }
        file_1.write_all(buf_1.as_slice()).unwrap();

        let result = join_all_pieces(
            piece_count,
            "target",
            "./src/download_manager/test_downloads/join/test_2",
        );

        match result {
            Ok(_) => panic!("Should have failed when trying to join all pieces"),
            Err(err) => assert!(matches!(err, DownloadManagerError::MissingPieceError(2))),
        }
    }

    #[test]
    fn joins_all_3_pieces_middle_file_missing_returns_error() {
        let piece_count = 3;

        let mut file_0 = File::create(format!(
            "./src/download_manager/test_downloads/join/test_3/pieces/0",
        ))
        .unwrap();
        let mut buf_0: Vec<u8> = Vec::new();
        for i in 0..100 {
            buf_0.push(i as u8);
        }
        file_0.write_all(buf_0.as_slice()).unwrap();

        let mut file_1 = File::create(format!(
            "./src/download_manager/test_downloads/join/test_3/pieces/2",
        ))
        .unwrap();
        let mut buf_1: Vec<u8> = Vec::new();
        for i in 0..100 {
            buf_1.push(i as u8);
        }
        file_1.write_all(buf_1.as_slice()).unwrap();

        let result = join_all_pieces(
            piece_count,
            "target",
            "./src/download_manager/test_downloads/join/test_3",
        );

        match result {
            Ok(_) => panic!("Should have failed when trying to join all pieces"),
            Err(err) => assert!(matches!(err, DownloadManagerError::MissingPieceError(1))),
        }
    }
}
