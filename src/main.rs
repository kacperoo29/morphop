mod image;

use gloo_events::EventListener;
use js_sys::Uint8Array;
use wasm_bindgen::JsCast;
use web_sys::{ImageData, HtmlInputElement};
use yew::prelude::*;

enum Msg {
    FileUpload(Event),
    FileLoaded(Vec<u8>),
    RadiusChanged(InputEvent),
    Dilate,
    Erode,
    Open,
    Close,
    HitAndMiss,
}

struct App {
    canvas: NodeRef,
    canvas_ctx: Option<web_sys::CanvasRenderingContext2d>,
    image_data: Option<Vec<u8>>,
    is_loading: bool,
    original_image: Option<image::Image>,
    image: Option<image::Image>,
    radius: u32,
}

impl Component for App {
    type Message = Msg;
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        let canvas = NodeRef::default();
        let canvas_ctx = None;
        let image_data = None;
        let is_loading = false;
        let original_image = None;
        let image = None;
        let radius = 5;

        Self {
            canvas,
            canvas_ctx,
            image_data,
            is_loading,
            original_image,
            image,
            radius,
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let canvas_width = if self.image.is_some() {
            self.image.as_ref().unwrap().get_width()
        } else {
            0
        };

        let canvas_height = if self.image.is_some() {
            self.image.as_ref().unwrap().get_height()
        } else {
            0
        };

        html! {
            <div>
                <div>
                    <input type="file" onchange={ctx.link().callback(|event: Event| Msg::FileUpload(event))} />
                    if self.is_loading {
                        <span>{"Loading image..."}</span>
                    }
                    if self.original_image.is_some() {
                        <input type="range" min="0" max="25" 
                            value={self.radius.to_string()} 
                            oninput={ctx.link().callback(|event: InputEvent| Msg::RadiusChanged(event))} />
                        <button onclick={ctx.link().callback(|_| Msg::Dilate)}>{"Dilate"}</button>
                        <button onclick={ctx.link().callback(|_| Msg::Erode)}>{"Erode"}</button>
                        <button onclick={ctx.link().callback(|_| Msg::Open)}>{"Open"}</button>
                        <button onclick={ctx.link().callback(|_| Msg::Close)}>{"Close"}</button>
                        <button onclick={ctx.link().callback(|_| Msg::HitAndMiss)}>{"Hit and Miss"}</button>
                    }
                </div>
                <canvas 
                    ref={self.canvas.clone()} 
                    width={canvas_width.to_string()} 
                    height={canvas_height.to_string()} />
                
            </div>
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::FileUpload(event) => {
                self.is_loading = true;
                let file_cb = ctx.link().callback(|value: Vec<u8>| Msg::FileLoaded(value));
                let target = event.target().unwrap();
                let target: web_sys::HtmlInputElement = target.dyn_into().unwrap();
                let file = target.files().unwrap().get(0).unwrap();
                let file_reader = web_sys::FileReader::new().unwrap();
                file_reader.read_as_array_buffer(&file).unwrap();
                let listener = EventListener::new(&file_reader, "load", move |event| {
                    let target = event.target().unwrap();
                    let target: web_sys::FileReader = target.dyn_into().unwrap();
                    let result = target.result().unwrap();
                    let array = Uint8Array::new(&result);

                    file_cb.emit(array.to_vec());
                });
                listener.forget();

                true
            }
            Msg::FileLoaded(data) => {
                self.is_loading = false;
                self.image_data = Some(data);
                self.original_image = Some(image::Image::new_with_data(
                    self.image_data.clone().unwrap(),
                ));
                self.image = Some(self.original_image.clone().unwrap());

                true
            }
            Msg::RadiusChanged(event) => {
                let target: HtmlInputElement = event.target().unwrap().dyn_into().unwrap();
                let value = target.value_as_number() as u32;
                self.radius = value;

                true
            },
            Msg::Dilate => {
                if let Some(image) = &self.image {
                    self.image = Some(image.dilate(self.radius));
                }

                true
            }
            Msg::Erode => {
                if let Some(image) = &self.image {
                    self.image = Some(image.erode(self.radius));
                }

                true
            }
            Msg::Open => {
                if let Some(image) = &self.image {
                    self.image = Some(image.open(self.radius));
                }

                true
            },
            Msg::Close => {
                if let Some(image) = &self.image {
                    self.image = Some(image.close(self.radius));
                }

                true
            },
            Msg::HitAndMiss => {
                if let Some(image) = &self.image {
                    self.image = Some(image.hit_and_miss(self.radius));
                }

                true
            }
        }
    }

    fn rendered(&mut self, _ctx: &Context<Self>, first_render: bool) {
        if first_render {
            let canvas = self.canvas.cast::<web_sys::HtmlCanvasElement>().unwrap();
            let ctx = canvas
                .get_context("2d")
                .unwrap()
                .unwrap()
                .dyn_into::<web_sys::CanvasRenderingContext2d>()
                .unwrap();

            self.canvas_ctx = Some(ctx);
        }

        if let Some(image) = &self.image {
            let ctx = self.canvas_ctx.as_ref().unwrap();
            let image_data = ImageData::new_with_u8_clamped_array_and_sh(
                wasm_bindgen::Clamped(&image.get_bitmap_data()),
                image.get_width(),
                image.get_height(),
            )
            .unwrap();

            ctx.clear_rect(
                0.0,
                0.0,
                image.get_width().into(),
                image.get_height().into(),
            );
            ctx.put_image_data(&image_data, 0.0, 0.0).unwrap();
        }
    }
}

fn main() {
    wasm_logger::init(wasm_logger::Config::default());
    yew::start_app::<App>();
}
