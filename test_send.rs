trait Boring: Send {}
fn is_send<T: Send + ?Sized>(_: &T) {}
fn test(b: &dyn Boring) {
    is_send(b);
}
fn main() {}
