use super::platform_impl::platform::library as platform;

pub struct Library(platform::Library);

impl Library {
    pub fn new<S: AsRef<::std::ffi::OsStr>>(filename: S) -> Library{
        let _lib = platform::Library::load(filename);
        Library(_lib)
    }

    pub fn get<'lib, T>(&'lib self, symbol: &str) -> T {
        unsafe {
            self.0.get(symbol)
        }
    }

    pub fn close(self) {
        self.0.close()
    }
}