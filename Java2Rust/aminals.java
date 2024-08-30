class Animal{
    private String habit;
    protected String sound;
    
    public Animal(){
        habit="";
        sound="";
    }
    
    void tick(){System.out.println(sound+" I like to "+getHabit()+".");}
    
    public String getHabit(){return habit;}
    public void setHabit(String habit){this.habit=habit;}
}

class Cat extends Animal{
    public Cat(){
        super();
        setHabit("crawl");
        sound="Meow";
    }
}

class PlayfulCat extends Cat{
    public PlayfulCat(){
        super();
        setHabit("play");
    }
}

public class Main {
    public static void main(String args[]) {
        
        Cat cat=new Cat();
        PlayfulCat playfulCat=new PlayfulCat();
        
        cat.tick();
        playfulCat.tick();
    }
}