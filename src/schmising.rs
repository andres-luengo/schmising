use rand::prelude::*;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Spin {
    Down,
    Up,
    OOB
}

pub struct MagnetizedMaterial<const W: usize, const H: usize>{
    pub cells: [[Spin; W]; H],
    pub temperature: f32,
    rng: ThreadRng
}

impl<const W: usize, const H: usize> MagnetizedMaterial<W, H> {
    pub fn new_halfhalf(temperature: f32) -> MagnetizedMaterial<W, H> {
        let mut mat = MagnetizedMaterial {
            cells: [[Spin::Up; W]; H],
            temperature,
            rng: rand::rng()
        };

        for r in 0..H {
            for c in W/2..W {
                mat.cells[r][c] = Spin::Down;
            }
        }

        mat
    }

    pub fn get_cell(&self, row: isize, col: isize) -> Spin {
        if row < 0 || row >= H as isize || col < 0 || col >= W as isize {
            Spin::OOB
        } else {
            self.cells[row as usize][col as usize]
        }
    }

    fn up_likelihood(&self, r: isize, c: isize) -> f32 {
        let mut up_count = 0;
        let mut down_count = 0;
        
        for dr in -1isize..=1 {
            for dc in -1isize..=1 {
                if dr == 0 && dc == 0 {
                    continue;
                }
                match self.get_cell(r as isize + dr, c as isize + dc) {
                    Spin::Up => up_count += 1,
                    Spin::Down => down_count += 1,
                    Spin::OOB => {}
                }
            }
        }

        let neighborhood_net_spin = (up_count - down_count) as f32 / (up_count + down_count) as f32;
        
        // this is where physics idk yet comes in. let's just do a logistic curve because a) i just learned more about them b) correct range and c) pretty easy to insert a temperature parameter
        // a more realistic model probably also doesn't look at only adjacent cells and maybe does some sort of integral of spin * gaussian envelope centered at cell or something.
        // also also i doubt that this probability doesn't depend on the cell itself's spin
        1. / (1. + (-neighborhood_net_spin / self.temperature).exp())
    }

    // returns true if a flip happened
    pub fn step(&mut self) -> bool {
        // there's gotta be a way to just make this function return an isize...
        let r = self.rng.random_range(0..H).try_into().unwrap();
        let c = self.rng.random_range(0..W).try_into().unwrap();

        let prob_up = self.up_likelihood(r, c);
        let roll = self.rng.random_range(0.0..=1.0);

        let current = self.cells[r as usize][c as usize];
        let result = if roll <= prob_up { Spin::Up } else { Spin::Down };
        self.cells[r as usize][c as usize] = result;

        current != result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_new_halfhalf() {
        let mat = MagnetizedMaterial::<4, 4>::new_halfhalf(1.0);
        for r in 0..4 {
            for c in 0..2 {
                assert_eq!(mat.cells[r][c], Spin::Up);
            }
            for c in 2..4 {
                assert_eq!(mat.cells[r][c], Spin::Down);
            }
        }
    }

    #[test]
    fn test_get_cell() {
        let mat = MagnetizedMaterial::<4, 4>::new_halfhalf(1.0);
        assert_eq!(mat.get_cell(0, 0), Spin::Up);
        assert_eq!(mat.get_cell(0, 3), Spin::Down);
        assert_eq!(mat.get_cell(-1, 0), Spin::OOB);
        assert_eq!(mat.get_cell(0, 4), Spin::OOB); 
    }

    #[test]
    fn test_up_likelihood() {
        let mat = MagnetizedMaterial::<4, 4>::new_halfhalf(1.0);
        println!("up_likelihood(0, 0) = {}", mat.up_likelihood(0, 0));
        assert!(mat.up_likelihood(0, 0) > 0.5); // should be 1.0 because all neighbors are up
    }
}