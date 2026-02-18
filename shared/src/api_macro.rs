#[macro_export]
macro_rules! api {
	(
		$(
			async fn $name:ident ( $($arg:ident : $arg_ty:ty),* ) -> $resp:ty ;
		)*
	) => {
		#[derive(Debug, Serialize, Deserialize)]
		#[serde(tag = "kind", content = "payload")]
		pub enum Request {
			$(
				#[allow(non_camel_case_types)]
				$name {
					$($arg : $arg_ty),*
				},
			)*
		}
		impl Request {
			pub fn get_name(&self) -> &'static str {
				match self {
					$(
						Self::$name { .. } => stringify!($name),
					)*
				}
			}
		}
		impl std::fmt::Display for Request {
			fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
				match self {
					$(
						Self::$name { $($arg),* } => {
							write!(f, "{}(", stringify!($name))?;

							#[allow(unused_assignments, unused)]
							let mut first = true;
							$(
								if !first { write!(f, ", ")?; }
								write!(f, "{}: {:?}", stringify!($arg), $arg)?;
								#[allow(unused_assignments)]
								{ first = false; }
							)*

							write!(f, ")")
						}
					)*
				}
			}
		}

		#[derive(Debug, Serialize, Deserialize)]
		#[serde(tag = "kind", content = "payload")]
		pub enum Response {
			$(
				#[allow(non_camel_case_types)]
				$name($resp),
			)*
		}

		#[async_trait::async_trait]
		pub trait Api {
			$(
				#[allow(non_snake_case)]
				async fn $name(&self, $($arg : $arg_ty),*) -> $resp;
			)*

			async fn dispatch(&self, request: Request) -> Response {
				match request {
					$(
						Request::$name{
							$($arg,)*
						} => {
							let result = self.$name($($arg),*).await;
							Response::$name(result)
						}
					)*
				}
			}
		}
	};
}
