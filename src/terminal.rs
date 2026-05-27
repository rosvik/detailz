use std::io::IsTerminal;

pub struct Size {
    pub width: usize,
    #[allow(dead_code)]
    pub height: usize,
}
const DEFAULT_SIZE: Size = Size {
    width: 80,
    height: 24,
};

pub fn terminal_size() -> Size {
    if !std::io::stdout().is_terminal() {
        return DEFAULT_SIZE;
    }

    let mut winsize = libc::winsize {
        ws_row: 0,
        ws_col: 0,
        ws_xpixel: 0,
        ws_ypixel: 0,
    };

    let result = unsafe {
        // SAFETY: `winsize` points to valid writable memory and stdout file descriptor is valid.
        libc::ioctl(libc::STDOUT_FILENO, libc::TIOCGWINSZ, &mut winsize)
    };

    if result == 0 && winsize.ws_col > 0 {
        Size {
            width: winsize.ws_col as usize,
            height: winsize.ws_row as usize,
        }
    } else {
        DEFAULT_SIZE
    }
}
