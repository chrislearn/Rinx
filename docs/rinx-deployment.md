# Rinx deployment and Octos ownership

Rinx has one client implementation and two application entry points. Build the
standalone desktop application with `cargo build --release --locked --bin rinx`.
OctoSense links the library with default features disabled and
`features = ["octosense-module"]`.

The standalone feature includes the OUP transport. The hosted feature does not
link `octos-app-transport` or `octos-core`; the shell injects a service instead.
Enabling both Cargo features also enables the standalone dependency, so the
final shell workspace must keep Rinx's default features disabled.

## Standalone configuration

Open **Discover → Mini apps → Developer**, then choose **Local kernel** or
**Remote server**. AI configuration currently lives in this developer screen.
Connecting explicitly starts the selected transport; it does not run a model
turn. Matrix and the article editor do not require an AI connection.

For a local kernel, select absolute paths to an Octos executable and its
configuration file, and enter its profile ID. Rinx launches that exact executable
as `serve --stdio --data-dir <Rinx data>/octos --config <selected file>`. There is
no shell command expansion or executable search on `PATH`. Configure the selected
Octos file/profile for your model provider; Rinx does not copy another Octos
installation's credentials or profiles. The kernel executable is supplied
separately in this implementation; release packaging does not yet bundle it.

For a remote kernel, enter the server URL, profile ID, transport access token and
an absolute workspace root **on the server**. The server must admit scoped
sessions beneath that root. Each mini app requests
`<server root>/<hex Matrix account>/<hex app ID>` as its workspace and read
allowlist. Provision these directories on the server if its workspace policy
requires existing directories. A rejected workspace remains an error; Rinx does
not retry with the server's unrestricted default workspace. Local UI storage is
not uploaded or treated as a remote filesystem path.

The selected mode and non-secret fields are saved atomically in
`<Rinx data>/octos-deployment.json` and restored into the configuration form.
Tokens stay in memory and are cleared from the input after a connection attempt.
After restarting, explicitly reconnect; remote mode requires the token again.
There is no new plaintext credential store or automatic connection on startup.

Closing a mini app revokes its requests while keeping the app's configured
transport available. Disconnect, provider replacement and Rinx shutdown revoke
old leases and close Rinx's transport. A local transport owns its child process;
a remote transport only disconnects and never requests server shutdown.

## OctoSense integration

The shell should register `rinx::module::HostedRinxModule::new(provider)`, where
`provider` is an `Arc<dyn rinx::octoscript_apps::OctosHost>`. This implements the
same Makepad `AppModule` contract as `RINX_MODULE`.

```rust
let module = rinx::module::HostedRinxModule::new(host_octos_services);
```

The host implements `OctosHost::open(&Lease, &Path)` and returns a scoped
`Arc<dyn OctosProvider>`. The lease contains the Matrix account, mini-app ID,
instance generation, optional room and granted services. The path is the local
UI workspace; the host decides the corresponding kernel request context.

The provider must enforce that lease for requests and asynchronous replies,
isolate mini-app request context, and implement cancellation/release without
stopping the system kernel. The host supplies its Rinx peer binding, shared
provider settings and model policy. This interface does not create peers itself.
It does not grant a mini app another app's transcript, tools or Matrix authority.

`RINX_MODULE` remains available for shells without a provider. It enters hosted
mode with AI unavailable, never searches for AppCard and never starts a local
kernel. Both module variants declare the four supported `octos.*` capabilities.
Hosted mode hides and rejects standalone connection setup. Closing the module
invalidates Rinx leases and releases the injected service; the shared runtime
remains owned by the shell.

## Integration still required

The OctoSense shell must register the injected module and implement the provider.
Its system-owned peer lifecycle, private memory namespaces and model selection
are the cross-repository work described in
[ADR 0007](adr/0007-host-owned-octos-app-peers.md). They are not supplied by the
former AppCard connection registry. Local kernel bundling, protected token
persistence and live remote-server/device qualification remain separate work.
