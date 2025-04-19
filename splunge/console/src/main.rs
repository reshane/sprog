use yew::prelude::*;
use gloo_net::http::Request;
use lib_grundit::types::{Comment, Note, User};

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

#[derive(Properties, PartialEq)]
struct NoteComponentProps {
    notes: Vec<Note>,
}

#[function_component(NoteComponent)]
fn note_component(NoteComponentProps { notes }: &NoteComponentProps) -> Html {
    notes.iter().map(|note| html!{
        <div style={"display: flex; padding: 5px;"}>
            <span style={ "display: flex; flex-direction: column; padding-left: 5px;" }>
                <span>{note.id}</span>
                <span>{format!("{}", note.contents)}</span>
                <a href={format!("/data/comment?byNoteId={}", note.id)}>{ "comments" }</a>
            </span>
        </div>
    }).collect()
}

#[derive(Properties, PartialEq)]
struct CommentComponentProps {
    comments: Vec<Comment>,
}

#[function_component(CommentComponent)]
fn comment_component(CommentComponentProps { comments }: &CommentComponentProps) -> Html {
    comments.iter().map(|comment| html!{
        <div style={"display: flex; padding: 5px;"}>
            <span style={ "display: flex; flex-direction: column; padding-left: 5px;" }>
                <span>{comment.id}</span>
                <span>{format!("{}", comment.contents)}</span>
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

    let notes = use_state(|| vec![]);
    {
        let notes = notes.clone();
        use_effect_with((), move |_| {
            let notes = notes.clone();
            wasm_bindgen_futures::spawn_local(async move {
                match Request::get("/data/note").send().await {
                    Ok(data) => {
                        match data.json::<Vec<Note>>().await {
                            Ok(json) => {
                                notes.set(json);
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

    let comments = use_state(|| vec![]);
    {
        let comments = comments.clone();
        use_effect_with((), move |_| {
            let comments = comments.clone();
            wasm_bindgen_futures::spawn_local(async move {
                match Request::get("/data/comment").send().await {
                    Ok(data) => {
                        match data.json::<Vec<Comment>>().await {
                            Ok(json) => {
                                comments.set(json);
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
            <div id={"navBar"} class={classes!("navBar")}>
                <button class={classes!("navLink", "active")}>{"Profile"}</button>
                <button class={classes!("navLink")}>{"Explore"}</button>
            </div>
            <a href={"/auth/google/login"} style={"padding: 5px;"}>{ "Login" }</a>
            <a href={"/auth/logout"} style={"padding: 5px;"}>{ "Logout" }</a>
            <div style={"display: flex; flex-direction: row;"}>
                <UserComponent users={(*users).clone()}/>
                <NoteComponent notes={(*notes).clone()}/>
                <CommentComponent comments={(*comments).clone()}/>
            </div>
        </div>
    }
}

fn main() {
    wasm_logger::init(wasm_logger::Config::default());
    yew::Renderer::<App>::new().render();
}
