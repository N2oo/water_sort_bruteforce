#[derive(Debug,Clone)]
pub struct Pipe {
    identifier:String,
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
    pub fn new(identifier:String) -> Self {
        Pipe {
            identifier: identifier,
            inner_colors: vec![],

        }
    }
    
    pub fn all_same_colors(&self)->bool{
        if self.inner_colors.len() < 1{
            return false;
        }
        let pickedColor = self.inner_colors.last().unwrap();
        for color in &self.inner_colors{
            if *pickedColor != *color{
                return false;
            }
        }
        return true;
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
        if destination.identifier == self.identifier{
            return false;
        }
        if self.is_empty() && destination.is_empty(){
            return false;
        }
        if self.all_same_colors() && destination.is_empty(){
            return false;
        }
        //SI LA DESTINATION EST VIDE RENVOYER TRUE
        if destination.is_empty() && !self.is_empty() {
            return true;
        }
        
        if destination.is_filled() || self.is_empty(){
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
    fn all_same_color_empty() {
        let source = Pipe::new(String::from("P1"));

        assert!(!source.all_same_colors());
    }

    #[test]
    fn all_same_color_one() {
        let mut source = Pipe::new(String::from("P1"));
        source.add_color(Color::Blue);

        assert!(source.all_same_colors());
    }
    #[test]
    fn all_same_color_two() {
        let mut source = Pipe::new(String::from("P1"));
        source.add_color(Color::Blue);
        source.add_color(Color::Blue);

        assert!(source.all_same_colors());
    }

    #[test]
    fn all_same_color_shuffle() {
        let mut source = Pipe::new(String::from("P1"));
        source.add_color(Color::Blue);
        source.add_color(Color::Blue);
        source.add_color(Color::Red);
        source.add_color(Color::Blue);

        assert!(!source.all_same_colors());
    }

    #[test]
    fn can_be_finished_because_all_same_color() {
        let mut source = Pipe::new(String::from("P1"));
        source.add_color(Color::Red);
        source.add_color(Color::Red);
        source.add_color(Color::Red);
        source.add_color(Color::Red);
        assert!(source.is_completed());
    }

    #[test]
    fn can_be_finished_because_empty() {
        let source = Pipe::new(String::from("P1"));
        assert!(source.is_completed());
    }

    #[test]
    fn can_be_not_finished_because_mixed() {
        let mut source = Pipe::new(String::from("P1"));
        source.add_color(Color::Red);
        source.add_color(Color::Red);
        source.add_color(Color::Red);
        source.add_color(Color::Blue);
        assert!(!source.is_completed());
    }
    #[test]
    fn can_be_not_finished_because_not_completed() {
        let mut source = Pipe::new(String::from("P1"));
        source.add_color(Color::Red);
        source.add_color(Color::Red);
        source.add_color(Color::Red);
        assert!(!source.is_completed());
    }

    #[test]
    fn can_pour_into_empty_pipe_because_multiple_src_color() {
        let mut source = Pipe::new(String::from("P1"));
        source.add_color(Color::Red);
        source.add_color(Color::Blue);

        let destination = Pipe::new(String::from("P2"));

        assert!(source.can_pour(&destination));
    }

    #[test]
    fn cannot_pour_into_empty_pipe_because_single_src_color() {
        let mut source = Pipe::new(String::from("P1"));
        source.add_color(Color::Red);

        let destination = Pipe::new(String::from("P2"));

        assert!(!source.can_pour(&destination));
    }

    #[test]
    fn cannot_pour_into_same_identifier() {
        let mut source = Pipe::new(String::from("P1"));
        source.add_color(Color::Red);
        
        let destination = Pipe::new(String::from("P1"));

        assert!(!source.can_pour(&destination));
    }

    #[test]
    fn cannot_pour_if_colors_different() {
        let mut source = Pipe::new(String::from("P1"));
        source.add_color(Color::Red);

        let mut destination = Pipe::new(String::from("P2"));
        destination.add_color(Color::Blue);
        assert!(!source.can_pour(&destination));
    }

    #[test]
    fn cannot_pour_if_both_empty() {
        let source = Pipe::new(String::from("P1"));

        let destination = Pipe::new(String::from("P2"));

        assert!(!source.can_pour(&destination));
    }

    #[test]
    fn cannot_pour_if_all_same_to_empty() {
        let mut source = Pipe::new(String::from("P1"));
        source.add_color(Color::Red);
        source.add_color(Color::Red);
        source.add_color(Color::Red);

        let destination = Pipe::new(String::from("P2"));

        assert!(!source.can_pour(&destination));
    }

    #[test]
    fn can_pour_if_same_color() {
        let mut source = Pipe::new(String::from("P1"));
        source.add_color(Color::Red);

        let mut destination = Pipe::new(String::from("P2"));
        destination.add_color(Color::Red);

        assert!(source.can_pour(&destination));
    }

    #[test]
    fn cannot_pour_if_destination_full() {
        let mut source = Pipe::new(String::from("P1"));
        source.add_color(Color::Red);

        let mut destination = Pipe::new(String::from("P2"));
        destination.add_color(Color::Red);
        destination.add_color(Color::Red);
        destination.add_color(Color::Red);
        destination.add_color(Color::Red);

        assert!(!source.can_pour(&destination));
    }

    #[test]
    fn cannot_pour_if_source_empty() {
        let source = Pipe::new(String::from("P1"));
        let destination = Pipe::new(String::from("P2"));

        assert!(!source.can_pour(&destination));
    }

    #[test]
    fn poor_if_dst_empty() {
        let mut source = Pipe::new(String::from("P1"));
        source.add_color(Color::Blue);
        source.add_color(Color::Red);
        let mut destination = Pipe::new(String::from("P2"));

        source.pour_into(&mut destination);
        assert_eq!(destination.get_filled_level(), 1);
        assert_eq!(source.get_filled_level(), 1);
    }

    #[test]
    fn poor_if_dst_good_color() {
        let mut source = Pipe::new(String::from("P1"));
        source.add_color(Color::Blue);
        source.add_color(Color::Red);
        let mut destination = Pipe::new(String::from("P2"));
        destination.add_color(Color::Blue);
        destination.add_color(Color::Red);

        source.pour_into(&mut destination);

        assert_eq!(source.get_filled_level(), 1);
        assert_eq!(destination.get_filled_level(), 3);
    }
    #[test]
    fn poor_if_dst_wrong_color() {
        let mut source = Pipe::new(String::from("P1"));
        source.add_color(Color::Blue);
        source.add_color(Color::Red);
        let mut destination = Pipe::new(String::from("P2"));
        destination.add_color(Color::Blue);
        destination.add_color(Color::Red);
        destination.add_color(Color::Yellow);

        source.pour_into(&mut destination);

        assert_eq!(source.get_filled_level(), 2);
        assert_eq!(destination.get_filled_level(), 3);
    }
    #[test]
    fn poor_if_src_empty() {
        let mut source = Pipe::new(String::from("P1"));
        let mut destination = Pipe::new(String::from("P2"));
        destination.add_color(Color::Blue);
        destination.add_color(Color::Red);
        destination.add_color(Color::Yellow);

        source.pour_into(&mut destination);

        assert_eq!(source.get_filled_level(), 0);
        assert_eq!(destination.get_filled_level(), 3);
    }
}
