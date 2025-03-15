use std::{cell::RefCell, rc::Rc};

use serde_json::Value;
use wasm_bindgen::prelude::*;
use web_sys::{
    HtmlCanvasElement, WebGlBuffer, WebGlProgram, WebGlRenderingContext, WebGlShader,
    WebGlUniformLocation, Window,
};
use rand::Rng;

struct NodeDrawContext {
    program: WebGlProgram,
    position_location: u32,
    node_position_location: Option<WebGlUniformLocation>,
    vertices: Vec<f32>,
    vertex_buffer: WebGlBuffer,
}

impl NodeDrawContext {
    fn new(context: &WebGlRenderingContext) -> Self {
        // build shader program
        let program = setup_shader_program(
            context,
            include_str!("../shaders/node/vert.glsl"),
            include_str!("../shaders/node/frag.glsl"),
        );
        // get locations
        let position_location = context.get_attrib_location(&program, "a_position") as u32;
        let node_position_location = context.get_uniform_location(&program, "u_node_position");
        // make vertex buffer for triangle fan
        let vertices = generate_circle_vertices(0.06, 30);
        let vertex_buffer = context.create_buffer().unwrap();
        populate_array_buffer(context, &vertex_buffer, &vertices);
        Self {
            program,
            position_location,
            node_position_location,
            vertices,
            vertex_buffer,
        }
    }

    fn draw(&self, context: &WebGlRenderingContext, nodes: &[(f32, f32)]) {
        context.use_program(Some(&self.program));
        context.bind_buffer(
            WebGlRenderingContext::ARRAY_BUFFER,
            Some(&self.vertex_buffer),
        );
        context.vertex_attrib_pointer_with_i32(
            self.position_location,
            2,
            WebGlRenderingContext::FLOAT,
            false,
            0,
            0,
        );
        context.enable_vertex_attrib_array(self.position_location);
        for (x, y) in nodes {
            context.uniform2f(Some(&self.node_position_location.clone().unwrap()), *x, *y);
            context.draw_arrays(
                WebGlRenderingContext::TRIANGLE_FAN,
                0,
                self.vertices.len() as i32 / 2,
            );
        }
    }
}

struct EdgeDrawContext {
    program: WebGlProgram,
    position_location: u32,
    vertices: Vec<f32>,
    vertex_buffer: WebGlBuffer,
}

impl EdgeDrawContext {
    fn new(
        context: &WebGlRenderingContext,
        nodes: &[(f32, f32)],
        edges: &[(usize, usize)],
    ) -> Self {
        // build shader program
        let program = setup_shader_program(
            context,
            include_str!("../shaders/edge/vert.glsl"),
            include_str!("../shaders/edge/frag.glsl"),
        );
        // get locations
        let position_location = context.get_attrib_location(&program, "a_position") as u32;
        // make vertex buffer for triangle fan
        let vertices = generate_edge_vertices(nodes, edges);
        let vertex_buffer = context.create_buffer().unwrap();
        populate_array_buffer(context, &vertex_buffer, &vertices);
        Self {
            program,
            position_location,
            vertices,
            vertex_buffer,
        }
    }

    fn update(&mut self, context: &WebGlRenderingContext, nodes: &[(f32, f32)], edges: &[(usize, usize)])  {
        self.vertices = generate_edge_vertices(nodes, edges);
        self.vertex_buffer = context.create_buffer().unwrap();
        populate_array_buffer(context, &self.vertex_buffer, &self.vertices);
    }

    fn draw(&self, context: &WebGlRenderingContext) {
        context.use_program(Some(&self.program));
        context.line_width(2.0);
        context.bind_buffer(
            WebGlRenderingContext::ARRAY_BUFFER,
            Some(&self.vertex_buffer),
        );
        context.vertex_attrib_pointer_with_i32(
            self.position_location,
            2,
            WebGlRenderingContext::FLOAT,
            false,
            0,
            0,
        );
        context.enable_vertex_attrib_array(self.position_location);
        context.draw_arrays(
            WebGlRenderingContext::LINES,
            0,
            self.vertices.len() as i32 / 2,
        );
    }
}

struct SiteGraph {
    nodes: Vec<(f32, f32)>,
    edges: Vec<(usize, usize)>,
    velocities: Vec<(f32, f32)>,
}

impl SiteGraph {
    fn from_json_str(json_str: &str) -> Self {
        let json: Value = serde_json::from_str(json_str).unwrap();
        let node_count = json["vizdata"]["node_count"].as_u64().unwrap() as usize;
        let edges_values = json["vizdata"]["edges"].as_array().unwrap();
        let edges = edges_values.iter().map(|v| {
            let edge_values = v.as_array().unwrap();
            let node_out = edge_values[0].as_u64().unwrap() as usize;
            let node_in = edge_values[1].as_u64().unwrap() as usize;
            (node_out, node_in)
        }).collect();
        Self {
            nodes: vec![(0.0, 0.0); node_count],
            edges,
            velocities: vec![(0.0, 0.0); node_count],
        }
    }

    fn initial_layout(&mut self, margin: f32) {
        let mut rng = rand::thread_rng();
        for node in &mut self.nodes {
            let x = rng.gen_range(-1.0 + margin..1.0 - margin);
            let y = rng.gen_range(-1.0 + margin..1.0 - margin);
            *node = (x, y);
        }
    }

    fn step(&mut self, repulsive_force_constant: f32, attractive_force_constant: f32, margin: f32, smoothing_factor: f32) {
        let mut forces = vec![(0.0, 0.0); self.nodes.len()];
        // repulsion
        for i in 0..self.nodes.len() {
            for j in i + 1..self.nodes.len() {
                let dx = self.nodes[i].0 - self.nodes[j].0;
                let dy = self.nodes[i].1 - self.nodes[j].1;
                let distance_sq = dx * dx + dy * dy;
                if distance_sq > 0.0 {
                    let distance = distance_sq.sqrt();
                    let force_magnitude = repulsive_force_constant / distance_sq;
                    forces[i].0 += force_magnitude * dx / distance;
                    forces[i].1 += force_magnitude * dy / distance;
                    forces[j].0 -= force_magnitude * dx / distance;
                    forces[j].1 -= force_magnitude * dy / distance;
                }
            }
        }
        // attraction
        for &(node_out, node_in) in &self.edges {
            if node_out != node_in {
                let dx = self.nodes[node_out].0 - self.nodes[node_in].0;
                let dy = self.nodes[node_out].1 - self.nodes[node_in].1;
                let distance = (dx * dx + dy * dy).sqrt();
                let force_magnitude = attractive_force_constant * distance;
                forces[node_out].0 -= force_magnitude * dx / distance;
                forces[node_out].1 -= force_magnitude * dy / distance;
                forces[node_in].0 += force_magnitude * dx / distance;
                forces[node_in].1 += force_magnitude * dy / distance;
            }
        }
        // position update
        for (i, (x, y)) in forces.iter_mut().enumerate() {
            self.velocities[i].0 = self.velocities[i].0  * (1.0 - smoothing_factor) + *x * smoothing_factor;
            self.velocities[i].1 = self.velocities[i].1  * (1.0 - smoothing_factor) + *y * smoothing_factor;
            self.nodes[i].0 += self.velocities[i].0;
            self.nodes[i].1 += self.velocities[i].1;
            self.nodes[i].0 = self.nodes[i].0.clamp(-1.0 + margin, 1.0 - margin);
            self.nodes[i].1 = self.nodes[i].1.clamp(-1.0 + margin, 1.0 - margin);
        }
    }
}

#[wasm_bindgen]
pub fn run() {
    let mut sitegraph = SiteGraph::from_json_str(include_str!("../../../sitegraph.json"));
    sitegraph.initial_layout(0.1);
    // get context
    let window = web_sys::window().unwrap();
    let context = get_context(&window);
    // initialize attributes
    let time = Rc::new(RefCell::new(0.0));
    // initialize draw contexts
    let node_draw_ctx = NodeDrawContext::new(&context);
    let mut edge_draw_ctx = EdgeDrawContext::new(&context, &sitegraph.nodes, &sitegraph.edges);
    // animation function
    let animate = Rc::new(RefCell::new(None::<Closure<dyn FnMut()>>));
    let animate_clone = animate.clone();
    let window_rc = Rc::new(RefCell::new(window));
    let window_rc_clone = window_rc.clone();
    let closure = Closure::wrap(Box::new(move || {
        // clear canvas
        context.clear_color(0.0, 0.0, 0.0, 0.0);
        context.clear(WebGlRenderingContext::COLOR_BUFFER_BIT);
        // update time
        let time_val = *time.borrow();
        *time.borrow_mut() = time_val + 0.01;
        sitegraph.step(0.0005, 0.0005, 0.1, 0.01);
        edge_draw_ctx.update(&context, &sitegraph.nodes, &sitegraph.edges);
        // draw graph
        node_draw_ctx.draw(&context, &sitegraph.nodes);
        edge_draw_ctx.draw(&context);
        // request next frame
        window_rc_clone
            .borrow()
            .request_animation_frame(
                animate_clone
                    .borrow()
                    .as_ref()
                    .unwrap()
                    .as_ref()
                    .unchecked_ref(),
            )
            .unwrap();
    }) as Box<dyn FnMut()>);
    animate.borrow_mut().replace(closure);
    // initiate draw
    window_rc
        .borrow()
        .request_animation_frame(animate.borrow().as_ref().unwrap().as_ref().unchecked_ref())
        .unwrap();
}

fn populate_array_buffer(context: &WebGlRenderingContext, buffer: &WebGlBuffer, data: &[f32]) {
    context.bind_buffer(WebGlRenderingContext::ARRAY_BUFFER, Some(buffer));
    unsafe {
        let data_arr = js_sys::Float32Array::view(data);
        context.buffer_data_with_array_buffer_view(
            WebGlRenderingContext::ARRAY_BUFFER,
            &data_arr,
            WebGlRenderingContext::STATIC_DRAW,
        );
    }
}

fn generate_edge_vertices(nodes: &[(f32, f32)], edges: &[(usize, usize)]) -> Vec<f32> {
    let mut edge_vertices = Vec::new();
    for &(start, end) in edges {
        edge_vertices.push(nodes[start].0);
        edge_vertices.push(nodes[start].1);
        edge_vertices.push(nodes[end].0);
        edge_vertices.push(nodes[end].1);
    }
    edge_vertices
}

fn generate_circle_vertices(radius: f32, segments: u32) -> Vec<f32> {
    let mut vertices = Vec::new();
    vertices.push(0.0);
    vertices.push(0.0);
    for i in 0..segments {
        let angle = 2.0 * std::f32::consts::PI * (i as f32) / (segments as f32);
        print!("{}", angle.cos());
        let x = radius * angle.cos();
        let y = radius * angle.sin();
        vertices.push(x);
        vertices.push(y);
    }
    vertices.push(vertices[2]);
    vertices.push(vertices[3]);
    vertices
}

fn setup_shader_program(
    context: &WebGlRenderingContext,
    vertex_shader_source: &str,
    fragment_shader_source: &str,
) -> WebGlProgram {
    let vertex_shader = setup_shader(
        context,
        WebGlRenderingContext::VERTEX_SHADER,
        vertex_shader_source,
    );
    let fragment_shader = setup_shader(
        context,
        WebGlRenderingContext::FRAGMENT_SHADER,
        fragment_shader_source,
    );
    let program = context.create_program().unwrap();
    context.attach_shader(&program, &vertex_shader);
    context.attach_shader(&program, &fragment_shader);
    context.link_program(&program);
    program
}

fn setup_shader(
    context: &WebGlRenderingContext,
    shader_type: u32,
    shader_source: &str,
) -> WebGlShader {
    let shader = context.create_shader(shader_type).unwrap();
    context.shader_source(&shader, shader_source);
    context.compile_shader(&shader);
    if context
        .get_shader_parameter(&shader, WebGlRenderingContext::COMPILE_STATUS)
        .is_falsy()
    {
        let info = context
            .get_shader_info_log(&shader)
            .unwrap_or_else(|| String::from("unknown error"));
        web_sys::console::log_1(&format!("could not compile shader: {}", info).into());
    }
    shader
}

fn get_context(window: &Window) -> WebGlRenderingContext {
    let document = window.document().unwrap();
    let canvas = document
        .get_element_by_id("graph-canvas")
        .unwrap()
        .dyn_into::<HtmlCanvasElement>()
        .unwrap();
    canvas
        .get_context("webgl")
        .unwrap()
        .unwrap()
        .dyn_into::<WebGlRenderingContext>()
        .unwrap()
}
