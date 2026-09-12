use std::collections::VecDeque;

pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

pub struct BeltlineProperties {
    pub min_item_distance: u32,
    pub speed: u32,
}

pub struct BeltSegment {
    pub properties: BeltlineProperties,
    pub length: u32,
    pub distances: VecDeque<u32>,
    pub distances_total: u32,
}

impl BeltSegment {
    fn new(properties: BeltlineProperties, length: u32) -> Self {
        let distances = VecDeque::new();
        let distances_total = 0;
        BeltSegment {
            properties,
            length,
            distances,
            distances_total,
        }
    }
    fn add_item_to_start(&mut self) -> Result<(),()> {
        let offset = self.length - self.distances_total;
        if offset >= self.properties.min_item_distance {
            self.distances.push_front(offset);
            self.distances_total += offset;
            Ok(())
        } else {
            Err(())
        }
    }
    fn tick(&mut self) {
        for i in 0..self.properties.speed {
            let reverse_distances = self.distances.iter_mut().rev();
            let mut skipped_reverse_distances = reverse_distances.skip_while(|d| d <= &&mut self.properties.min_item_distance);
            if let Some(d) = skipped_reverse_distances.next() {
                *d -= 1;
                self.distances_total -= 1;
            }
        }
    }
    fn print(&self) {
        println!("BeltSegment: length: {}, distances_total: {}, distances: {:?}", self.length, self.distances_total, self.distances);
    }
}




#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_belt_segment() {
        let properties = BeltlineProperties {
            min_item_distance: 2,
            speed: 1,
        };
        let mut segment = BeltSegment::new(properties, 10);
        assert_eq!(segment.add_item_to_start(), Ok(()));
        segment.print();
        segment.tick();
        segment.print();
        segment.tick();
        segment.print();
        segment.tick();
        segment.print();
        segment.tick();
        segment.add_item_to_start();
        segment.print();
        segment.tick();
        segment.print();
        segment.tick();
        segment.print();
        segment.tick();
        segment.print();
        segment.tick();
        segment.print();
        segment.tick();
        segment.print();
        segment.tick();
        segment.print();
        segment.tick();
        segment.print();
        segment.add_item_to_start();
        segment.tick();
        segment.print();
        segment.tick();
        segment.print();
        segment.tick();
        segment.print();
        segment.tick();
        segment.print();
        segment.tick();
        segment.print();
        segment.tick();
        segment.print();
        segment.tick();
        segment.print();
        segment.tick();
        segment.print();
    }


    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
