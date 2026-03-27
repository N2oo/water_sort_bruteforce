use std::fs::OpenOptions;

#[derive(Debug)]
pub struct Pipe {
    inner_colors: Vec<Color>,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Color {
    Grey,
    Blue,
    Lemon,
    Brown,
    Green,
    Red,
    LightGreen,
    LightBlue,
    Pink,
    Orange,
    Purple,
    Yellow,
}

impl Pipe {
    pub fn new() -> Self {
        Pipe {
            inner_colors: vec![],
        }
    }

    pub fn is_empty(&self) -> bool {
        return self.inner_colors.is_empty();
    }
    pub fn is_filled(&self) -> bool {
        return self.inner_colors.len() >= 4;
    }
    pub fn get_filled_level(&self) -> usize {
        return self.inner_colors.len();
    }
    pub fn is_completed(&self) -> bool {
        // Cas 1 : vide → OK
        if self.inner_colors.is_empty() {
            return true;
        }

        // Cas 2 : pas 4 éléments → pas OK
        if self.inner_colors.len() != 4 {
            return false;
        }

        // Cas 3 : toutes les couleurs doivent être identiques
        let first = &self.inner_colors[0];

        self.inner_colors.iter().all(|c| c == first)
    }

    pub fn add_color(&mut self, color: Color) {
        self.inner_colors.push(color);
    }

    pub fn pour_into(&mut self, destination: &mut Pipe) {
        while self.can_pour(destination) {
            let color = self.inner_colors.pop().unwrap().clone();
            destination.add_color(color);
        }
    }

    pub fn can_pour(&self, destination: &Pipe) -> bool {
        //SI LA DESTINATION EST VIDE RENVOYER TRUE
        if destination.is_empty() && !self.is_empty() {
            return true;
        }
        if destination.is_filled() || self.is_empty() {
            return false;
        }

        return match (self.inner_colors.last(), destination.inner_colors.last()) {
            (Some(src), Some(dst)) => src == dst,
            _ => false,
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_be_finished_because_all_same_color() {
        let mut source = Pipe::new();
        source.add_color(Color::Red);
        source.add_color(Color::Red);
        source.add_color(Color::Red);
        source.add_color(Color::Red);
        assert!(source.is_completed());
    }

    #[test]
    fn can_be_finished_because_empty() {
        let source = Pipe::new();
        assert!(source.is_completed());
    }

    #[test]
    fn can_be_not_finished_because_mixed() {
        let mut source = Pipe::new();
        source.add_color(Color::Red);
        source.add_color(Color::Red);
        source.add_color(Color::Red);
        source.add_color(Color::Blue);
        assert!(!source.is_completed());
    }
    #[test]
    fn can_be_not_finished_because_not_completed() {
        let mut source = Pipe::new();
        source.add_color(Color::Red);
        source.add_color(Color::Red);
        source.add_color(Color::Red);
        assert!(!source.is_completed());
    }

    #[test]
    fn can_spill_into_empty_pipe() {
        let mut source = Pipe::new();
        source.add_color(Color::Red);

        let destination = Pipe::new();

        assert!(source.can_pour(&destination));
    }

    #[test]
    fn cannot_spill_if_colors_different() {
        let mut source = Pipe::new();
        source.add_color(Color::Red);

        let mut destination = Pipe::new();
        destination.add_color(Color::Blue);
        assert!(!source.can_pour(&destination));
    }

    #[test]
    fn can_spill_if_same_color() {
        let mut source = Pipe::new();
        source.add_color(Color::Red);

        let mut destination = Pipe::new();
        destination.add_color(Color::Red);

        assert!(source.can_pour(&destination));
    }

    #[test]
    fn cannot_spill_if_destination_full() {
        let mut source = Pipe::new();
        source.add_color(Color::Red);

        let mut destination = Pipe::new();
        destination.add_color(Color::Red);
        destination.add_color(Color::Red);
        destination.add_color(Color::Red);
        destination.add_color(Color::Red);

        assert!(!source.can_pour(&destination));
    }

    #[test]
    fn cannot_spill_if_source_empty() {
        let source = Pipe::new();
        let destination = Pipe::new();

        assert!(!source.can_pour(&destination));
    }

    #[test]
    fn poor_if_dst_empty() {
        let mut source = Pipe::new();
        source.add_color(Color::Blue);
        source.add_color(Color::Blue);
        let mut destination = Pipe::new();

        source.pour_into(&mut destination);

        assert!(source.is_empty());
        assert_eq!(destination.get_filled_level(), 2)
    }

    #[test]
    fn poor_if_dst_good_color() {
        let mut source = Pipe::new();
        source.add_color(Color::Blue);
        source.add_color(Color::Red);
        let mut destination = Pipe::new();
        destination.add_color(Color::Blue);
        destination.add_color(Color::Red);

        source.pour_into(&mut destination);

        assert_eq!(source.get_filled_level(), 1);
        assert_eq!(destination.get_filled_level(), 3);
    }
    #[test]
    fn poor_if_dst_wrong_color() {
        let mut source = Pipe::new();
        source.add_color(Color::Blue);
        source.add_color(Color::Red);
        let mut destination = Pipe::new();
        destination.add_color(Color::Blue);
        destination.add_color(Color::Red);
        destination.add_color(Color::Yellow);

        source.pour_into(&mut destination);

        assert_eq!(source.get_filled_level(), 2);
        assert_eq!(destination.get_filled_level(), 3);
    }
    #[test]
    fn poor_if_src_empty() {
        let mut source = Pipe::new();
        let mut destination = Pipe::new();
        destination.add_color(Color::Blue);
        destination.add_color(Color::Red);
        destination.add_color(Color::Yellow);

        source.pour_into(&mut destination);

        assert_eq!(source.get_filled_level(), 0);
        assert_eq!(destination.get_filled_level(), 3);
    }
}
