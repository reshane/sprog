use yew::prelude::*;
use serde::Deserialize;
use gloo_net::http::Request;

#[derive(Properties, Clone, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
struct User {
    id: i64,
    guid: String,
    name: String,
    email: String,
    picture: String,
}

#[derive(Properties, PartialEq)]
struct UserComponentProps{
    users: Vec<User>,
}

#[function_component(UserComponent)]
fn user_component(UserComponentProps { users }: &UserComponentProps) -> Html {
    users.iter().map(|user| html!{
        <div style={"display: flex; padding: 5px;"}>
            <img src={user.picture.clone()}/>
            <span style={ "display: flex; flex-direction: column; padding-left: 5px;" }>
                <span>{user.id}</span>
                <span>{format!("{}", user.name)}</span>
                <span>{format!("{}", user.guid)}</span>
                <span>{format!("{}", user.email)}</span>
                <a href={format!("/data/note?byOwnerId={}", user.id)}>{ "notes" }</a>
            </span>
        </div>
    }).collect()
}

#[function_component]
fn App() -> Html {
    let users = use_state(|| vec![]);
    {
        let users = users.clone();
        use_effect_with((), move |_| {
            let users = users.clone();
            wasm_bindgen_futures::spawn_local(async move {
                match Request::get("/data/user").send().await {
                    Ok(data) => {
                        match data.json::<Vec<User>>().await {
                            Ok(json) => {
                                users.set(json);
                            },
                            Err(e) => {
                                log::error!("{:?}", e);
                            },
                        }
                    },
                    Err(e) => {
                        log::error!("{:?}", e);
                    },
                }
            });
        });
    }

    html! {
        <div>
            <a href="/auth/google/login" style="padding: 5px;">{ "Login" }</a>
            <a href="/auth/logout" style="padding: 5px;">{ "Logout" }</a>
            <UserComponent users={(*users).clone()}/>
        </div>
    }
}

fn main() {
    wasm_logger::init(wasm_logger::Config::default());
    yew::Renderer::<App>::new().render();
}
