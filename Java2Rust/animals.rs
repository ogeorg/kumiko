trait AnimalHabit {
    fn habit(&self) -> &'static str {
        ""
    }
}

trait AnimalSound {
    fn sound(&self) -> &'static str {
        ""
    }
}

trait Animal: AnimalHabit + AnimalSound {
    fn tick(&self) {
        println!("{}. I like to {}.", self.sound(), self.habit());
    }
}

struct Cat;
impl Animal for Cat {}
impl AnimalHabit for Cat {
    fn habit(&self) -> &'static str {
        "crawl"
    }
}
impl AnimalSound for Cat {
    fn sound(&self) -> &'static str {
        "meow"
    }
}

struct PlayfulCat;
impl Animal for PlayfulCat {}
impl AnimalHabit for PlayfulCat {
    fn habit(&self) -> &'static str {
        "play"
    }
}
impl AnimalSound for PlayfulCat {
    fn sound(&self) -> &'static str {
        "meow"
    }
}

struct Turtle;
impl Animal for Turtle {
    fn tick(&self) {
        println!("go away!");
    }
}
impl AnimalHabit for Turtle {}
impl AnimalSound for Turtle {}

fn tick(animal: impl Animal) {
    animal.tick();
}

fn main() {
    tick(Cat);
    tick(PlayfulCat);
    tick(Turtle);
}
