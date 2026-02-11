// Original: https://github.com/jsjolund/yew-resize-split-view/blob/master/src/split.rs
// Modified a bit + updated dependencies

use gloo::events::EventListener;
use gloo::utils::window;
use stylist::Style;
use web_sys::wasm_bindgen::JsCast;
use web_sys::wasm_bindgen::prelude::Closure;
use web_sys::{DomRect, HtmlElement};
use yew::prelude::*;
use yew::virtual_dom::VNode;
use yew_hooks::use_local_storage;

#[derive(Clone, Copy, PartialEq)]
pub enum Axis {
	Vertical,
	Horizontal,
}

impl Axis {
	fn resize_dir(&self) -> &str {
		match self {
			Axis::Vertical => "width",
			Axis::Horizontal => "height",
		}
	}
	fn flex_dir(&self) -> &str {
		match self {
			Axis::Vertical => "row",
			Axis::Horizontal => "column",
		}
	}
	fn cursor(&self) -> &str {
		match self {
			Axis::Vertical => "col-resize",
			Axis::Horizontal => "row-resize",
		}
	}
	fn rect(&self, rect: &DomRect) -> i32 {
		match self {
			Axis::Vertical => rect.width() as i32,
			Axis::Horizontal => rect.height() as i32,
		}
	}
}

#[derive(Clone, Properties, PartialEq)]
pub struct Props {
	/// unique name to store last width in local storage
	pub name: AttrValue,
	pub axis: Axis,
	#[prop_or_default]
	pub height: Option<String>,
	/// of the left/top element
	#[prop_or_default]
	pub starting_width: Option<i32>,

	#[prop_or_default]
	pub left: Option<VNode>,
	#[prop_or_default]
	pub right: Option<VNode>,
	#[prop_or_default]
	pub top: Option<VNode>,
	#[prop_or_default]
	pub bottom: Option<VNode>,
}

// TODO: make left/top side width absolute (pixels, not percentage)
#[function_component]
pub fn SplitPanes(props: &Props) -> Html {
	let container = use_node_ref();
	let drag_area = use_node_ref();

	let is_resizing = use_mut_ref(|| false);
	let x = use_mut_ref(|| 0);
	let y = use_mut_ref(|| 0);
	let left_width_storage =
		use_local_storage::<i32>(format!("split_panes_{}_left_width", props.name));
	let left_width = use_mut_ref(|| {
		left_width_storage
			.clone()
			.unwrap_or(props.starting_width.unwrap_or(400))
	});
	let container_width = use_mut_ref(|| 0);

	let stopped_resizing = use_state(|| false);
	let new_left_width = use_state(|| *left_width.borrow_mut());

	let mut left_style = format!("{}: {}px;", props.axis.resize_dir(), *new_left_width);
	let mut right_style = String::from("flex: 1 1 0%;");
	let mut container_style = format!(
		"display: flex; flex: 1 1 0%; flex-direction: {}; {}: 100%;",
		props.axis.flex_dir(),
		props.axis.resize_dir()
	);

	if *is_resizing.borrow_mut() {
		let style = format!(
			"user-select:none; pointer-events:none; cursor:{};",
			props.axis.cursor()
		);
		left_style.push_str(&style);
		right_style.push_str(&style);
	}
	if let Some(height) = &props.height {
		let height_style = format!("max-height:{height};min-height:{height};height:{height};");
		container_style.push_str(height_style.as_str());
		left_style.push_str(height_style.as_str());
		right_style.push_str(height_style.as_str());
	}

	let left_css = Style::new(left_style).expect("Failed to create left style");
	let right_css = Style::new(right_style).expect("Failed to create right style");
	let container_css = Style::new(container_style).expect("Failed to create cont. style");

	{
		// Create window mouse up listener
		let container_width = container_width.clone();
		let container = container.clone();
		let props = props.clone();
		use_effect_with(container, move |container| {
			let div = container
				.cast::<HtmlElement>()
				.expect("drag not attached to div element");
			let listener = EventListener::new(&window(), "pointerup", move |_| {
				*container_width.borrow_mut() = props.axis.rect(&div.get_bounding_client_rect());
			});
			move || {
				drop(listener);
			}
		});
	}
	{
		// Create window resize listener
		let container_width = container_width.clone();
		let container = container.clone();
		let props = props.clone();
		use_effect_with(container, move |container| {
			let div = container
				.cast::<HtmlElement>()
				.expect("drag not attached to div element");
			let listener = EventListener::new(&window(), "resize", move |_| {
				*container_width.borrow_mut() = props.axis.rect(&div.get_bounding_client_rect());
			});
			move || {
				drop(listener);
			}
		});
	}
	{
		// Create mouse down listener
		let x = x.clone();
		let y = y.clone();
		let drag = drag_area.clone();
		let left_width = left_width.clone();
		let is_resizing = is_resizing.clone();
		let props = props.clone();
		use_effect_with(drag, move |drag| {
			let drag = drag.clone();
			let div = drag
				.cast::<HtmlElement>()
				.expect("drag not attached to div element");
			let left_side = div.previous_element_sibling().unwrap();
			let listener =
				Closure::<dyn Fn(PointerEvent)>::wrap(Box::new(move |ev: PointerEvent| {
					*x.borrow_mut() = ev.client_x();
					*y.borrow_mut() = ev.client_y();
					let size = props.axis.rect(&left_side.get_bounding_client_rect());
					*left_width.borrow_mut() = size;
					*is_resizing.borrow_mut() = true;
					let div = drag
						.cast::<HtmlElement>()
						.expect("drag not attached to div element");
					div.set_pointer_capture(ev.pointer_id())
						.expect("failed to capture (mouse) pointer on pointerdown");
				}));
			div.add_event_listener_with_callback("pointerdown", listener.as_ref().unchecked_ref())
				.unwrap();
			move || {
				drop(listener);
			}
		});
	}
	{
		// Create mouse move listener
		let container = container.clone();
		let is_resizing = is_resizing.clone();
		let new_left_width = new_left_width.clone();
		let props = props.clone();
		use_effect_with(container, move |container| {
			let div = container
				.cast::<HtmlElement>()
				.expect("container not attached to div element");
			if *container_width.borrow_mut() == 0 {
				*container_width.borrow_mut() = props.axis.rect(&div.get_bounding_client_rect());
			}
			let listener =
				Closure::<dyn Fn(PointerEvent)>::wrap(Box::new(move |ev: PointerEvent| {
					if *is_resizing.borrow_mut() {
						let dx = ev.client_x() - *x.borrow_mut();
						let dy = ev.client_y() - *y.borrow_mut();
						let d = match props.axis {
							Axis::Vertical => dx,
							Axis::Horizontal => dy,
						};
						let w = *left_width.borrow_mut() + d;
						new_left_width.set(w);
						left_width_storage.set(w);
					}
				}));
			div.add_event_listener_with_callback("pointermove", listener.as_ref().unchecked_ref())
				.unwrap();
			move || {
				drop(listener);
			}
		});
	}
	{
		// Create mouse up listener
		let container = container.clone();
		let drag = drag_area.clone();
		use_effect_with((container, drag), move |(container, drag)| {
			let drag = drag.clone();
			let drag_div = drag.cast::<HtmlElement>().expect("drag not div element");
			let div = container
				.cast::<HtmlElement>()
				.expect("container not attached to div element");
			let listener = Closure::<dyn Fn(PointerEvent)>::wrap(Box::new(move |ev| {
				*is_resizing.borrow_mut() = false;
				stopped_resizing.set(true);
				drag_div
					.release_pointer_capture(ev.pointer_id())
					.expect("failed to release pointer capture on pointerup");
			}));
			div.add_event_listener_with_callback("pointerup", listener.as_ref().unchecked_ref())
				.unwrap();
			move || {
				drop(listener);
			}
		});
	}

	let container_class_name = container_css.get_class_name().to_string();

	// width/height of the area that dragging is allowed
	let drag_area_size = "24px";
	// width/height of the visible line
	let drag_line_stroke_width = "1px";

	let (first_id, second_id, drag_id) = match props.axis {
		Axis::Vertical => ("left", "right", "vertical"),
		Axis::Horizontal => ("top", "bottom", "horizontal"),
	};
	let (first_html, second_html) = match props.axis {
		Axis::Vertical => (props.left.clone(), props.right.clone()),
		Axis::Horizontal => (props.top.clone(), props.bottom.clone()),
	};
	let (drag_area_width, drag_area_height) = match props.axis {
		Axis::Vertical => (drag_area_size, "100%"),
		Axis::Horizontal => ("100%", drag_area_size),
	};
	let (drag_line_width, drag_line_height) = match props.axis {
		Axis::Vertical => (drag_line_stroke_width, "100%"),
		Axis::Horizontal => ("100%", drag_line_stroke_width),
	};
	let cursor_style = props.axis.cursor();

	// TODO: remove horizontal axis or truly generalize this implementation -> see width: {}, height: 100%
	html! {
		<div ref={container} class={container_class_name}>
			<div class={format!("panel {}", left_css.get_class_name())} id={first_id}>
				{ first_html }
			</div>

			<div
				id={drag_id}
				ref={drag_area}
				class="drag group relative z-10 bg-[#555]"
				style={format!("cursor:{}; width: {}; height: {}; display: flex; flex-direction: {}; justify-content: center;", cursor_style, drag_line_width, drag_line_height, props.axis.flex_dir())}
			>
				<div style={format!("position: absolute; width: {}; height: {}; display: flex; flex-direction: {}; justify-content: center;", drag_area_width, drag_area_height, props.axis.flex_dir())}>
					<div style={format!("width: {}; height: {};", drag_area_width, drag_area_height)} />
				</div>
			</div>

			<div class={format!("panel {}", right_css.get_class_name())} style="width: 100%" id={second_id}>
				{ second_html }
			</div>
		</div>
	}
}
