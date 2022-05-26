use super::errors::DownloadManagerError;
use super::types::Piece;
use std::fs::File;
use std::io::Write;

// Creates downloads directory if it doesn't exist
fn create_downloads_directory(path: &str) -> Result<(), DownloadManagerError> {
    if !path.is_empty() && !std::path::Path::new(path).exists() {
        std::fs::create_dir_all(path)
            .map_err(|_| DownloadManagerError::CreateDirectoryError(path.to_string()))?;
    }

    Ok(())
}

#[allow(dead_code)]
/// Saves in disk a non-empty piece in disk in the specified path with the number of piece as file name
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
///     size_in_bytes: 3,
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
    create_downloads_directory(downloads_dir_path)?;

    let mut file = File::create(format!("{}/{}", downloads_dir_path, piece.piece_number))?;
    file.write_all(&piece.data[..])?;
    Ok(())
}

mod tests {

    #[allow(unused_imports)]
    use super::*;
    #[allow(unused_imports)]
    use std::io::Read;

    #[test]
    fn redundant_piece_test() {
        assert_eq!(1 + 1, 2);
    }

    #[test]
    fn saves_little_piece_in_disk_content_is_correct() {
        let piece_number = 1;
        let piece_length: u64 = 1;
        let mut piece = Piece {
            piece_number,
            data: Vec::new(),
            size_in_bytes: piece_length,
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
            size_in_bytes: piece_length,
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
        let piece_length: u64 = 654;
        let piece = Piece {
            piece_number,
            data: Vec::new(),
            size_in_bytes: piece_length,
        };

        let downloads_dir_path = "./src/download_manager/test_downloads";
        let result = save_piece_in_disk(&piece, downloads_dir_path);
        match result {
            Ok(_) => panic!("Should have failed when trying to save piece with no data"),
            Err(err) => assert!(matches!(err, DownloadManagerError::EmptyPieceError)),
        };
    }
}
