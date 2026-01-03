use axum::response::Redirect;

pub async fn portal_handler() -> Redirect {
    // Redirect to the frontend Next.js portal on port 3001
    Redirect::to("http://localhost:3001/auth/portal")
}
