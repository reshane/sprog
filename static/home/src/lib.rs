use wasm_bindgen::prelude::*;
use std::{cell::RefCell, fmt::{Display, Formatter}, rc::Rc};
use web_sys::{console, HtmlCanvasElement, Position};
use lib_grundit::types::{Punch, RequestPunch, User};
use wasm_bindgen_futures::{spawn_local, JsFuture};
use web_sys::{Request, RequestInit, RequestMode, Response};

const CELL_SIZE: u32 = 5;
const GRID_COLOR: &'static str = "#CCCCCC";
const DEAD_COLOR: &'static str  = "#FFFFFF";
const ALIVE_COLOR: &'static str = "#000000";

fn window() -> web_sys::Window {
    web_sys::window().expect("should have a window in this context")
}

fn request_animation_frame(f: &Closure<dyn FnMut()>) {
    window()
    .request_animation_frame(f.as_ref().unchecked_ref())
        .expect("should be able to request animation frame");
}

fn document() -> web_sys::Document {
    window().document().expect("window should have a document")
}

fn canvas() -> web_sys::HtmlCanvasElement {
    let canvas = document().get_element_by_id("canvas").unwrap();
    canvas
        .dyn_into::<web_sys::HtmlCanvasElement>()
        .map_err(|_| ())
        .unwrap()
}

pub async fn whoami() -> Result<JsValue, JsValue> {
    let opts = RequestInit::new();
    opts.set_method("GET");
    opts.set_mode(RequestMode::Cors);

    let url = format!("/data/whoami");

    let request = Request::new_with_str_and_init(&url, &opts)?;

    request
        .headers()
        .set("Accept", "application/vnd.github.v3+json")?;

    let window = web_sys::window().unwrap();
    let resp_value = JsFuture::from(window.fetch_with_request(&request)).await?;

    // `resp_value` is a `Response` object.
    assert!(resp_value.is_instance_of::<Response>());
    let resp: Response = resp_value.dyn_into().unwrap();

    // Convert this other `Promise` into a rust `Future`.
    let json = JsFuture::from(resp.json()?).await?;

    // Send the JSON response back to JS.
    Ok(json)
}

async fn get_user() -> Result<User, ()> {
    if let Ok(user_json) = whoami().await {
        match serde_wasm_bindgen::from_value(user_json) {
            Ok(user) => Ok(user),
            Err(e) => {
                console::error_1(&format!("{:?}", e).into());
                Err(())
            }
        }
    } else {
        Err(())
    }
}

pub async fn post_punch(punch: RequestPunch) -> Result<JsValue, JsValue> {
    let options = wasm_request::get_options::<RequestPunch>(
        "data/punch",
        wasm_request::Method::POST,
        None,
        Some(wasm_request::DataType::Json(punch)),
    );

    match wasm_request::request(options).await {
        Ok(resp) => Ok(resp),
        Err(e) => {
            console::error_1(&format!("{:?}", e).into());
            Err(JsValue::null())
        },
    }
}

pub async fn post_punch_1(punch: RequestPunch) {
    let ans = post_punch(punch).await;
    console::log_1(&format!("{:?}", ans).into());
}

#[wasm_bindgen]
pub async fn run() -> Result<(), JsValue> {
    let f = Rc::new(RefCell::new(None));
    let user = Rc::new(RefCell::new(None));

    {
        match get_user().await {
            Ok(user_data) => {
                *user.borrow_mut() = Some(user_data);
            },
            Err(()) => {},
        }
    }

    let position_callback = Closure::wrap(Box::new(move |position: Position| {
        console::log_1(&format!("{:?}", user).into());
        if let Some(ref user_data) = *user.borrow() {
            let punch = RequestPunch {
                id: None,
                owner_id: Some(user_data.id),
                geo: Some(format!("timestamp: {}, lat: {} lon: {}", position.timestamp(), position.coords().latitude(), position.coords().longitude())),
            };
            console::log_1(&format!("{:?}", &punch).into());
            spawn_local(post_punch_1(punch));
        }
    }) as Box<dyn FnMut(Position)>);

    if let Ok(geo) = window().navigator().geolocation() {
        let _ = geo.get_current_position(position_callback.as_ref().unchecked_ref());
    }

    position_callback.forget();

    let g = f.clone();
    {
        let mut universe = Universe::new();
        let canvas = canvas();
        canvas.set_attribute("width", format!("{}", (CELL_SIZE + 1) * universe.width() + 1).as_str())?;
        canvas.set_attribute("height", format!("{}", (CELL_SIZE + 1) * universe.height() + 1).as_str())?;
        *g.borrow_mut() = Some(Closure::wrap(Box::new(move || {
            universe.tick();
            draw_grid(&canvas, universe.width(), universe.height());
            draw_cells(&canvas, &universe);
            request_animation_frame(f.borrow().as_ref().unwrap());
        }) as Box<dyn FnMut()>));
    }

    request_animation_frame(g.borrow().as_ref().unwrap());

    Ok(())
}

fn draw_grid(canvas: &HtmlCanvasElement, width: u32, height: u32) {
    let ctx = canvas.get_context("2d")
        .unwrap()
        .unwrap()
        .dyn_into::<web_sys::CanvasRenderingContext2d>()
        .unwrap();
    ctx.begin_path();
    ctx.set_stroke_style_str(GRID_COLOR);

    // Vertical lines
    for i in 0..width {
        ctx.move_to((i * (CELL_SIZE + 1) + 1) as f64, 0_f64);
        ctx.line_to((i * (CELL_SIZE + 1) + 1) as f64, ((CELL_SIZE + 1) * height + 1) as f64);
    }

    // Horizontal lines
    for i in 0..height {
        ctx.move_to(0_f64, (i * (CELL_SIZE + 1) + 1) as f64);
        ctx.line_to(((CELL_SIZE + 1) * width + 1) as f64, (i * (CELL_SIZE + 1) + 1) as f64);
    }

    ctx.stroke();
}

fn draw_cells(canvas: &HtmlCanvasElement, universe: &Universe) {
    let (width, height) = (universe.width(), universe.height());
    let ctx = canvas.get_context("2d")
        .unwrap()
        .unwrap()
        .dyn_into::<web_sys::CanvasRenderingContext2d>()
        .unwrap();
    ctx.begin_path();

    for row in 0..height {
        for col in 0..width {
            let idx = universe.get_idx(row as usize, col as usize);
            ctx.set_fill_style_str(match universe.cells[idx] {
                Cell::Dead => DEAD_COLOR,
                Cell::Alive => ALIVE_COLOR,
            });

            ctx.fill_rect(
                (col * (CELL_SIZE + 1) + 1) as f64,
                (row * (CELL_SIZE + 1) + 1) as f64,
                CELL_SIZE as f64, CELL_SIZE as f64
            );
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Cell {
    Dead = 0,
    Alive = 1,
}

struct Universe {
    width: usize,
    height: usize,
    cells: Vec<Cell>,
}

impl Display for Universe {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        for line in self.cells.as_slice().chunks(self.width as usize) {
            for &cell in line {
                let symbol = if cell == Cell::Alive { '◻' } else { '◼' };
                write!(f, "{}", symbol)?;
            }
            write!(f, "\n")?;
        }
        Ok(())
    }
}

impl Universe {
    fn get_idx(&self, row: usize, col: usize) -> usize {
        row * self.width + col
    }

    fn live_neighbor_count(&self, row: usize, col: usize) -> u8 {
        let mut count = 0;
        for d_row in [self.height - 1, 0, 1].iter().cloned() {
            for d_col in [self.width - 1, 0, 1].iter().cloned() {
                if d_row == 0 && d_col == 0 {
                    continue;
                }

                let nbor_row = (d_row + row) % self.height;
                let nbor_col = (d_col + col) % self.width;
                let idx = self.get_idx(nbor_row, nbor_col);
                count += self.cells[idx] as u8;
            }
        }
        count
    }

    pub fn tick(&mut self) {
        let mut next = self.cells.clone();
        
        for row in 0..self.height {
            for col in 0..self.width {
                let idx = self.get_idx(row, col);
                let cell = self.cells[idx];
                let live_nbors = self.live_neighbor_count(row, col);
                next[idx] = match (cell, live_nbors) {
                    (Cell::Alive, x) if x < 2 => Cell::Dead,
                    (Cell::Alive, 2) | (Cell::Alive, 3) => Cell::Alive,
                    (Cell::Alive, x) if x > 3 => Cell::Dead,
                    (Cell::Dead, 3) => Cell::Alive,
                    (current, _) => current,
                }
            }
        }

        self.cells = next;
    }

    pub fn new() -> Self {
        let width = 128;
        let height = 64;
        let cells = (0..width * height)
            .map(|i| {
                if i % 2 == 0 || i % 7 == 0 {
                    Cell::Alive
                } else {
                    Cell::Dead
                }
            }).collect();
        Universe {
            width,
            height,
            cells,
        }
    }

    pub fn width(&self) -> u32 {
        self.width as u32
    }

    pub fn height(&self) -> u32 {
        self.height as u32
    }
}

