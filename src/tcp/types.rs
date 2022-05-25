pub trait TcpConnection {
    fn write(&mut self, data: &[u8]);

    fn read(&mut self, buf: &mut Vec<u8>);
}
