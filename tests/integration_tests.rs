use task_hookrs::task::Task;
use task_streamer::schema::Topic;
use task_streamer::app::{AppState, app_config};
use actix_web::{rt as actix_rt, test, web, App};

use serde_json;

fn fake_tasks() -> Vec<Task> {
    let task_json = r#"
    [{"status":"pending","uuid":"d3c2052f-31b5-4544-94bc-af3ef1b10c4b","entry":"20201118T071926Z","description":"figure out frontend static asset storage/serving","tags":["@stream","@home"],"id":13,"modified":"20201118T071926Z","project":"twitch.task-display","brainpower":"M","urgency":2.15205},{"status":"pending","uuid":"8699cf59-59d4-4f42-812d-0d2de0cad191","entry":"20201120T064735Z","description":"add tests","tags":["@home","@stream"],"id":14,"modified":"20201230T070650Z","project":"twitch.task-display","brainpower":"M","urgency":2.1411},{"status":"pending","uuid":"6c2b9f0f-10a2-4e36-8f13-c160e7dbc3cb","entry":"20201125T062735Z","description":"structured logging","tags":["@home","@stream"],"id":15,"modified":"20201125T062735Z","project":"twitch.task-display","brainpower":"M","urgency":2.1137},{"status":"pending","uuid":"02cb9bfc-fa96-4293-a71e-b833ca3e8795","entry":"20201127T085138Z","description":"figure out js build pipeline","tags":["@home","@stream"],"id":16,"modified":"20201127T085138Z","project":"twitch.task-display","brainpower":"M","urgency":2.09726},{"status":"pending","uuid":"d1b4a190-d4fc-4d8f-aa88-a53398a8b17c","entry":"20201230T074345Z","description":"fix styling everywhere","tags":["@home","@stream"],"id":17,"modified":"20201230T074345Z","project":"twitch.task-display","brainpower":"M","urgency":1.91644},{"status":"pending","uuid":"4e57c902-ea40-4bf0-b459-7226366a23da","entry":"20201230T074402Z","description":"figure out button position and icon","tags":["@home","@stream"],"id":18,"modified":"20201230T074402Z","project":"twitch.task-display","brainpower":"M","urgency":1.91644},{"status":"pending","uuid":"086ac9f7-9650-4aa8-8232-bcb84773b0e6","entry":"20201230T074421Z","description":"make side of screen drawer is on configurable","tags":["@home","@stream"],"id":19,"modified":"20210101T074359Z","project":"twitch.task-display","brainpower":"M","urgency":1.91644},{"status":"pending","uuid":"56bdf1ee-77fc-4c98-8779-54d97a118c41","entry":"20210103T060110Z","description":"display controls on hover","tags":["@home","@stream","next"],"id":20,"modified":"20210103T060110Z","project":"twitch.task-display","brainpower":"M","urgency":17}]
"#;
    serde_json::from_str(task_json).unwrap()
}

#[actix_rt::test]
async fn listsing_tasks() {
    let state = web::Data::new(AppState::new("Foo bar baz".to_string()));
    let mut app = test::init_service(
        App::new()
            .app_data(state.clone())
            .configure(app_config)
    ).await;

    let req = test::TestRequest::with_header("content-type", "application/json").uri("/api/v1/tasks").to_request();
    let resp = test::call_service(&mut app, req).await;
    assert!(resp.status().is_success());

    let tasks: Vec<Task> = test::read_body_json(resp).await;
    assert!(tasks.is_empty());

    {
        let mut t = state.tasks.lock().unwrap();
        *t = fake_tasks();
    }

    let req = test::TestRequest::with_header("content-type", "application/json").uri("/api/v1/tasks").to_request();
    let resp = test::call_service(&mut app, req).await;
    assert!(resp.status().is_success());

    let tasks: Vec<Task> = test::read_body_json(resp).await;
    assert_eq!(tasks, fake_tasks());
}

#[actix_rt::test]
async fn setting_tasks() {
    let state = web::Data::new(AppState::new("Foo bar baz".to_string()));

    let mut app = test::init_service(
        App::new()
            .app_data(state.clone())
            .configure(app_config)
    ).await;

    {
        assert_eq!(*state.tasks.lock().unwrap(), vec![]);
    }

    let req = test::TestRequest::post()
        .header("content-type", "application/json")
        .header("Authorization", "Bearer Foo bar baz")
        .uri("/api/v1/tasks")
        .set_json(&fake_tasks())
        .to_request();

    let resp = test::call_service(&mut app, req).await;
    assert!(resp.status().is_success());

    {
        assert_eq!(*state.tasks.lock().unwrap(), fake_tasks());
    }
}

#[actix_rt::test]
async fn getting_topic() {
    let state = web::Data::new(AppState::new("Foo bar baz".to_string()));

    let mut app = test::init_service(
        App::new()
            .app_data(state.clone())
            .configure(app_config)
    ).await;

    let req = test::TestRequest::with_header("content-type", "application/json").uri("/api/v1/topic").to_request();
    let resp = test::call_service(&mut app, req).await;
    assert!(resp.status().is_success());

    let topic: Topic = test::read_body_json(resp).await;
    assert_eq!(topic, Topic::default());

    let expected = Topic::new("herp".to_string(), "derp".to_string());

    {
        let mut t = state.topic.lock().unwrap();
        *t = expected.clone();
    }

    let req = test::TestRequest::with_header("content-type", "application/json").uri("/api/v1/topic").to_request();
    let resp = test::call_service(&mut app, req).await;
    assert!(resp.status().is_success());

    let topic: Topic = test::read_body_json(resp).await;
    assert_eq!(topic, expected);
}

#[actix_rt::test]
async fn setting_topic() {
    let state = web::Data::new(AppState::new("Foo bar baz".to_string()));

    let mut app = test::init_service(
        App::new()
            .app_data(state.clone())
            .configure(app_config)
    ).await;

    let expected = Topic::new("herp".to_string(), "derp".to_string());

    let req = test::TestRequest::post()
        .header("content-type", "application/json")
        .header("Authorization", "Bearer Foo bar baz")
        .uri("/api/v1/topic")
        .set_json(&expected)
        .to_request();

    let resp = test::call_service(&mut app, req).await;
    assert!(resp.status().is_success());

    {
        assert_eq!(*state.topic.lock().unwrap(), expected);
    }
}
