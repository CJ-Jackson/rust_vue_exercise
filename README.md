# Rust Vue Exercise

I just wanted to try out a [Rocket Web framework](https://rocket.rs) and [Vue.js](https://vuejs.org/). I will start with
the framework first, I thought it was exceptional, I mostly like
the [request guard feature](https://api.rocket.rs/v0.5/rocket/request/trait.FromRequest), I used that to build my own
dependency injection system with support for feature flags. The feature flags are there to handle the edge cases, for
example:

* Which constructor to call?
* Which database connection to use?
* Which authentication system to use?
* Which user level to allow?
* Are visitors allowed to see the page?

The code for dependency injection is in `src/dependency.rs` and `src/user/dependency.rs`, to make the service compatible
with DI, it just needs to implement one of the two traits (or both).

```rust
pub trait FromGlobalContext: Sized {
    fn from_global_context<'r>(
        dependency_global_context: &'r DependencyGlobalContext<'r, '_>,
        flag: Arc<DependencyFlagData>,
    ) -> impl Future<Output=Result<Self, DependencyError>> + Send;
}

pub trait FromUserContext: Sized {
    fn from_user_context<'r>(
        dependency_user_context: &'r DependencyUserContext<'r, '_>,
        flag: Arc<DependencyFlagData>,
    ) -> impl Future<Output=Result<Self, DependencyError>> + Send;
}
```

The `DependencyGlobalContext` is the context that is available to all requests, it holds the configuration and the
database connection. The `DependencyUserContext` is the superset of `DependencyGlobalContext` and hold information about
the current user. The `DependencyFlagData` is the data used to determine which constructor to call or which database
connection to use.

The most interesting example of how I used DI is in `src/html_base.rs` and to inject the dependency into the route
handler function with either `Dep<T, F = DefaultFlag>` (Global Context) or `UserDep<T, F = DefaultFlag>` (Plus User
Context and check permissions).

```rust
#[get("/")]
fn index(context_html_builder: UserDep<ContextHtmlBuilder>) -> Markup {
    let title = "Hello, world!".to_string();
    context_html_builder.0.attach_title(title.clone()).attach_content(html! { h1 { (title) } }).build();
}
```

The [maud's HTML](https://maud.lambda.xyz/) macro compliment vue.js quite nicely.

```rust
#[get("/")]
async fn root(context_html_builder: UserDep<ContextHtmlBuilder>) -> Markup {
    let title = "Rust Vue Exercise";
    context_html_builder
        .0
        .attach_title(title.to_string())
        .set_current_tag("home".to_string())
        .attach_content(html! {
            h1 .mt-3 { (title) }
            p .mt-3 { "This is Rust Vue Exercise." }
            h2 .mt-3 { "Exercise 1" }
            div #app .mt-3 v-cloak { "{{ message }}" }
            h2 .mt-3 { "Exercise 2" }
            div #counter .mt-3 v-cloak {
                button .btn .btn-sky-blue "@click"="count++" {
                    "Count is: {{ count }}  "
                    (plus_icon())
                }
            }
            h2 .mt-3 { "Exercise 3" }
            div #array .mt-3 v-cloak {
                ul .ul-bullet {
                    li "v-for"="(item) in items" { "{{ item }}" }
                }
            }
        })
        .attach_footer(root_js())
        .build()
}
```

The HTML macro is a bit like writing in CSS, I find it more natural than actually writing HTML. It does not get in the
way of Vue.js syntax at all, if I used a different template system, like Twig for example, I would've used the
`verbatim` feature as it shares the same syntax as vue.js.

The mental mind switching between HTML macro, CSS, Rust and JavaScript; it is quite seamless as it resembles each other
with the C syntax. I can see how it would improve the productivity of a developer, especially when working with
front-end frameworks.

Yes, I did use the Builder Pattern on HTML, I'd never thought I do that, but it is quite nice to have and I used DI on
that as well, just awesome. `ContextHtmlBuilder` it handles flash messages and user login too.

Rust does not allow null values (you can use `Option<T>` to allow null), and because it serializes nicely into JSON, it
actually enables fearless JavaScript in the front-end.

Rust is also quite a decent bundler, thanks to the macro like `include_str!`, I can embed the JavaScript code in the
binary, same for CSS and SQL.

```rust
fn main() {
    let css = include_str!("../public/css/app.css");
    let js = include_str!("../public/js/app.js");
    let sql = include_str!("../public/sql/schema.sql");
}
```
