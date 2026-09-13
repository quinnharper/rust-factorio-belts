// pub mod inventory;
pub mod belt;
pub(crate) mod polyfill;
pub mod util;

pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::belt::{BeltLine, BeltLineProperties};

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }

    #[test]
    fn belt_display() {
        let mut belt = BeltLine::new(BeltLineProperties::default());

        let tick_count = 5;
        for _ in 0..tick_count {
            belt.append_item(5, ());
        }

        let mut out = String::new();
        let mut countdown = tick_count - 1;
        while !belt.can_sleep() {
            belt.tick();
            belt.ascii_art_display_to(&mut out);
            println!("{}", out);
            countdown -= 1;
        }
        assert_eq!(countdown, 0);
    }
}
