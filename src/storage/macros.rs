#[macro_export]
macro_rules! send_asyncs {
    ($($vis:vis async fn $name:ident($($arg:ident: $arg_ty:ty),*) -> $ret:ty;)*) => {
            $(
                $vis fn $name($($arg: $arg_ty),*) -> impl Future<Output = $ret> + Send;
            )*
        };
    ($($vis:vis async fn $name:ident(&self $(, $arg:ident: $arg_ty:ty)*) -> $ret:ty;)*) => {
            $(
                $vis fn $name(&self $(, $arg: $arg_ty)*) -> impl Future<Output = $ret> + Send;
            )*
        };
    ($($vis:vis async fn $name:ident(&mut self $(, $arg:ident: $arg_ty:ty)*) -> $ret:ty;)*) => {
            $(
                $vis fn $name(&mut self $(, $arg: $arg_ty)*) -> impl Future<Output = $ret> + Send;
            )*
        };
    ($($vis:vis async fn $name:ident(self $(, $arg:ident: $arg_ty:ty)*) -> $ret:ty;)*) => {
            $(
                $vis fn $name(self $(, $arg: $arg_ty)*) -> impl Future<Output = $ret> + Send;
            )*
        };
}
