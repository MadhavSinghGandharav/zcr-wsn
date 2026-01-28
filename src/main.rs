
use macroquad::prelude::*;
use zcr_wsn::{config::SENSOR_RADIUS, node::Node};


struct SIMULATOR{
    wsn: Vec<Node>,
    round: usize,
    alive_count: usize
}

impl SIMULATOR{

    fn new(width: f32, height: f32, n_nodes: usize) -> Self{
        let wsn = Node::create_wsn(width, height, n_nodes);
        
        Self{
            wsn,
            round:0,
            alive_count:n_nodes
            
        }
    }

    fn render(self){
        for node in self.wsn.iter(){
            let color: Color = if node.is_alive {
                BEIGE
            }else{
                RED
            };

            draw_circle(node.position.x, node.position.y,SENSOR_RADIUS,color);
        }

    }


}

#[macroquad::main("WSN")]
async fn main() {
    loop{
        clear_background(BLACK);
        draw_text("WSN", screen_width()/2.0, screen_height()/2.0, 100.0, WHITE);
        next_frame().await
    }    
}
