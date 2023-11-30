use yew::prelude::*;

fn main() {
    yew::Renderer::<App>::new().render();
}

#[derive(Clone)]
struct TrackRecord { 
    id: String,
    image: String,
}

#[function_component(App)]
pub fn app() -> Html {
    let mut tracks: Vec<TrackRecord> = vec![
        TrackRecord { 
            id: "Track1".to_string(),
            image: "https://avatars.githubusercontent.com/u/15838364?v=4".to_string(),
        },
    ];


    for _ in 1..5 { 
        tracks.push(tracks.get(0).unwrap().clone());  
    }



    let tracks = tracks.iter().map(|track| html!{
        <img class="image" src={format!("{}",track.image)} draggable="false"/>
    }).collect::<Html>();
    // <div id="image-track" data-mouse-down-at="0" data-prev-percentage="0">
    //{ tracks }
    //</div>

    html! {
        <body>
            <TitleBar />
            <MediaControl/>
        </body>
    }
}


#[function_component]
fn MediaControl() -> Html {
    html!{ 
        <div id="bottom-bar"> 
            <button class="media-control-button"> <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 16 16"><path fill="none" stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5" d="M2.75 13.25L11.25 8l-8.5-5.25zm11.5-9.5v8.5"/></svg></button>
            <button class="media-control-button"> <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 16 16"><path fill="none" stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5" d="M2.75 2.75v10.5L12.25 8z"/></svg> </button>
            <button class="media-control-button"> <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 16 16"><path fill="none" stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5" d="M2.75 13.25L11.25 8l-8.5-5.25zm11.5-9.5v8.5"/></svg></button>
        
            <div class="progress-container">
                <span class="progressbar" style="width:60%"></span>
            </div>
        </div>
    }
}

#[function_component]
fn TitleBar() -> Html { 
    html! {
    <div class="titlebar">
        <div class="titlebar-button" id="titlebar-minimize">
            <img
            src="https://api.iconify.design/mdi:window-minimize.svg"
            alt="minimize"
            />
        </div>
        <div class="titlebar-button" id="titlebar-maximize">
            <img
            src="https://api.iconify.design/mdi:window-maximize.svg"
            alt="maximize"
            />
        </div>
        <div class="titlebar-button" id="titlebar-close">
            <img src="https://api.iconify.design/mdi:close.svg" alt="close" />
        </div>
    </div>
    }
}
