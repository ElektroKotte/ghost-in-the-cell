use std::io;
use std::fmt::Write;

/**** Parsing help ****/

macro_rules! print_err {
    ($($arg:tt)*) => (
        {
            use std::io::Write;
            writeln!(&mut ::std::io::stderr(), $($arg)*).ok();
        }
    )
}

macro_rules! parse_input {
    ($x:expr, $t:ident) => ($x.trim().parse::<$t>().unwrap())
}

fn get_line() -> String {
    let mut buffer = String::new();
    io::stdin().read_line(&mut buffer).unwrap();
    return buffer;
}

fn get_splitline() -> Vec<String> {
    get_line().split_whitespace().map(|tok| String::from(tok)).collect::<Vec<_>>()
}

/**** Controlling constants ****/

const BORDER_LIMIT: isize = 3;

/**** Game state helpers ****/

enum Command {
    Move { src: usize, dst: usize, bots: isize },
}

impl From<Command> for String {
    fn from(cmd: Command) -> Self {
        let mut out = String::new();
        match cmd {
            Command::Move { src, dst, bots } => {
                write!(&mut out, "MOVE {} {} {}", src, dst, bots);
            }
        }
        return out;
    }
}

struct Troop {
    owner: isize,
    bots: isize,
    turns: isize,
}

struct Factory {
    id: usize,
    owner: isize,
    bots: isize,
    production: isize,
    incoming: Vec<Troop>,
    distances: Vec<isize>,
}

impl Factory {
    fn new(id: usize, count: usize) -> Factory {
        Factory {
            id: id,
            owner: 0,
            bots: 0,
            production: 0,
            incoming: Vec::new(),
            distances: vec![std::isize::MAX; count],
        }
    }
    
    fn clear(&mut self) {
        self.incoming.clear();
    }

    fn is_interesting(&self) -> bool {
        self.production > 0
    }
}

struct GameState {
    factories: Vec<Factory>,
}

impl GameState {
    fn new(count: usize) -> GameState {
        let mut state = GameState {
            factories: Vec::new(),
        };
        for i in 0..count {
            state.factories.push(Factory::new(i, count));
        }
        return state;
    }
    
    fn clear(&mut self) {
        for ref mut f in self.factories.iter_mut() {
            f.clear();
        }
    }

    /**
     * Should return a list of indexes for the factories that should
     * targets for operations this turn. This include owned factories
     * that should be upgraded, are under attack. */
    fn targets(&self) -> Vec<usize> {
        let mut trgs = Vec::new();

        /* First step, try to identify bordering factories */
        for f in self.factories.iter().filter(|&f| f.owner > 0) {
            let dist = f.distances.iter().min().unwrap() * BORDER_LIMIT;
            let mut closest = f.distances.iter().enumerate().filter_map(
                |(i, &d)| {
                    if d < dist && self.factories[i].is_interesting() {Some(i)}
                    else {None}
                }).collect::<Vec<usize>>();
            trgs.append(&mut closest);
        }

        /* Above algorithm didn't give any target, likely early in game, use fallback */
        if trgs.len() == 0 {
            let mut all = self.factories.iter().filter_map(|ref f| {
                if f.is_interesting() { Some(f.id) }
                else { None }
            }).collect::<Vec<usize>>();
            trgs.append(&mut all);
        }

        /* And add all owned factories as they are always targets */
        let mut my = self.factories.iter().filter_map(|ref f| {
            if f.owner > 0 { Some(f.id) }
            else { None }
        }).collect::<Vec<usize>>();
        trgs.append(&mut my);

        /* Sort and remove duplicates */
        trgs.sort();
        trgs.dedup();

        return trgs;
    }

    fn moves_recursive(&self, targets: &Vec<usize>, i: usize, mut moves: Vec<Command>) -> Vec<Command> {
        if i >= targets.len() {
            return Vec::new();
        } else {
            // TODO return best of do a move, or not do a move
            let j = targets[i];
            if self.factories[j].owner <= 0 {
                let new_moves = self.moves_recursive(&targets, i + 1, Vec::new());
                moves.push(Command::Move {src: self.my_closest_to(j).unwrap(), dst: j, bots: 2});
            }
        }
        return moves;
    }

    fn moves(&self, targets: Vec<usize>) -> Vec<Command> {
        return self.moves_recursive(&targets, 0, Vec::new());
    }

    fn my_closest_to(&self, target: usize) -> Option<usize> {
        self.factories.iter()
            .filter(|ref f| f.owner > 0)
            .min_by_key(|ref f| f.distances[target])
            .map(|ref f| f.id)
    }

    fn closest_interesting_to(&self, target: usize) -> Option<usize> {
        self.factories.iter()
            .filter(|ref f| f.is_interesting())
            .min_by_key(|ref f| f.distances[target])
            .map(|ref f| f.id)
    }
}

fn main() {
    let factory_count = parse_input!(get_line(), usize);
    let link_count = parse_input!(get_line(), usize); 
    
    let mut state = GameState::new(factory_count);
    
    for _ in 0..link_count {
        let inputs = get_splitline();
        let factory_1 = parse_input!(inputs[0], usize);
        let factory_2 = parse_input!(inputs[1], usize);
        let distance = parse_input!(inputs[2], isize);
        
        state.factories[factory_1].distances[factory_2] = distance;
        state.factories[factory_2].distances[factory_1] = distance;
    }

    /* Game loop */
    loop {
        /* Create clean slate for this turn */
        state.clear();
        let mut commands: Vec<Command> = Vec::new();
        
        /* Build game state */
        let entity_count = parse_input!(get_line(), usize);
        for _ in 0..entity_count {
            let inputs = get_splitline();
            let id = parse_input!(inputs[0], usize);
            match inputs[1].as_str() {
                "FACTORY" => {
                    state.factories[id].owner = parse_input!(inputs[2], isize);
                    state.factories[id].bots = parse_input!(inputs[3], isize);
                    state.factories[id].production = parse_input!(inputs[4], isize);
                }
                "TROOP" => {
                    let target = parse_input!(inputs[4], usize);
                    state.factories[target].incoming.push( Troop {
                        owner: parse_input!(inputs[2], isize),
                        bots: parse_input!(inputs[5], isize),
                        turns: parse_input!(inputs[6], isize),
                    });
                }
                _ => /* BAD PROGRAM! Don't do that! */ ()
            }
        }

        let targets = state.targets();
        let mut commands_new = state.moves(targets);

        if commands.len() > 0 {
            let output: String = commands.into_iter().map(String::from).collect::<Vec<String>>().join(";");
            println!("{}", output);
        } else {
            println!("WAIT");
        }
    }
}
