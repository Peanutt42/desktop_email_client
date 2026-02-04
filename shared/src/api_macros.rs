//! macros to have a one source of thruth for tauri commands and there type signatures

#[cfg(feature = "frontend")]
pub mod invoke {
	use serde::{Serialize, de::DeserializeOwned};
	use wasm_bindgen::{JsValue, prelude::wasm_bindgen};

	#[wasm_bindgen]
	extern "C" {
		#[wasm_bindgen(js_namespace = ["window", "__TAURI__", "core"])]
		pub(crate) async fn invoke(cmd: &str, args: JsValue) -> JsValue;

		#[wasm_bindgen(js_namespace = ["window", "__TAURI__", "core"], js_name = invoke)]
		async fn invoke_without_args(cmd: &str) -> JsValue;
	}

	pub async fn invoke_typed<Args, Output>(cmd: &str, args: &Args) -> Output
	where
		Args: Serialize + std::fmt::Debug,
		Output: DeserializeOwned,
	{
		tracing::info!("{}({:?})", cmd, args);

		let args_js_value = serde_wasm_bindgen::to_value(args)
			.expect("failed to serialize paramters for tauri command using serde");
		let output_js_value = invoke(cmd, args_js_value).await;

		serde_wasm_bindgen::from_value(output_js_value)
			.expect("failed to deserialize tauri command's output using serde")
	}

	pub async fn invoke_typed_without_args<Output>(cmd: &str) -> Output
	where
		Output: DeserializeOwned,
	{
		tracing::info!("{}()", cmd);

		let output_js_value = invoke_without_args(cmd).await;
		serde_wasm_bindgen::from_value(output_js_value)
			.expect("failed to deserialize tauri command's output using serde")
	}
}

#[macro_export]
macro_rules! tauri_backend_api {
    (
        $(
            fn $name:ident ( $( $arg:ident : $ty:ty ),* $(,)? ) -> $ret:ty ;
        )*
    ) => {
        #[cfg(feature = "backend")]
        pub trait BackendApi: Send + Sync + 'static {
            $(
                $crate::tauri_backend_api!(@trait_fn fn $name ( $( $arg : $ty ),* ) -> $ret );
            )*
        }

        #[cfg(feature = "backend")]
        #[macro_export]
        macro_rules! register_tauri_commands {
            ($backend_ty:ty) => {{
                // Generate a module containing actual #[tauri::command] fns.
                // They forward into a managed backend instance.
                mod __tauri_commands {
                    use super::*;
                    use tauri::State;

                    $(
                        $crate::tauri_backend_api!(@command_fn fn $name ( $( $arg : $ty ),* ) -> $ret );
                    )*
                }

                tauri::generate_handler![
                    $(
                        __tauri_commands::$name
                    ),*
                ]
            }};
        }

        #[cfg(feature = "frontend")]
        $(
            $crate::tauri_backend_api!(@frontend_fn fn $name ( $( $arg : $ty ),* ) -> $ret );
        )*
    };

    (@trait_fn fn $name:ident ( $( $arg:ident : $ty:ty ),* ) -> $ret:ty ) => {
        fn $name(&self, $( $arg : $ty ),* ) -> $ret;
    };

    (@command_fn fn $name:ident ( $( $arg:ident : $ty:ty ),* ) -> $ret:ty ) => {
        #[tauri::command(rename_all = "snake_case")]
        pub fn $name(
            backend: State<'_, Box<dyn BackendApi>>,
            $( $arg : $ty ),*
        ) -> $ret {
            backend.$name($( $arg ),*)
        }
    };


    (@frontend_fn fn $name:ident () -> $ret:ty ) => {
        #[cfg(feature = "frontend")]
        pub async fn $name() -> $ret {
            $crate::api_macros::invoke::invoke_typed_without_args(stringify!($name)).await
        }
    };
    (@frontend_fn fn $name:ident ( $( $arg:ident : $ty:ty ),+ ) -> $ret:ty ) => {
        #[cfg(feature = "frontend")]
        pub async fn $name($( $arg : $ty ),*) -> $ret {
            #[derive(serde::Serialize, Debug)]
            struct Args { $( $arg: $ty, )* }
            let args = Args { $( $arg, )* };
            $crate::api_macros::invoke::invoke_typed(stringify!($name), &args).await
        }
    };
}
