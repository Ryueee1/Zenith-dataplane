# Zenith Dashboard

Modern web-based control center for Zenith Data Plane.

## Features

- **Real-time Monitoring**: Live updates every 2 seconds
- **Node Management**: View all registered data plane nodes
- **Plugin Registry**: Track deployed WASM plugins
- **Deployment Status**: Monitor plugin deployments
- **Metrics Dashboard**: System statistics and throughput

## Screenshots

### Main Dashboard
- Live node count
- Plugin registry status
- Active deployments
- Real-time throughput

### Tables
- Data Nodes with status indicators
- Plugin versions and URLs
- Deployment history

## Running

### Development
```bash
# Start control plane first
cd control-plane
cargo run

# Open dashboard
open dashboard/index.html
# or
python3 -m http.server 8000 --directory dashboard
```

Then navigate to `http://localhost:8000`

### Production
Serve via nginx or any static file server:

```nginx
server {
    listen 80;
    server_name dashboard.zenith.io;
    root /path/to/zenith-dataplane/dashboard;
    index index.html;
}
```

## API Integration

Dashboard connects to Control Plane API at `http://localhost:9090`

To change the API endpoint, edit `CONTROL_PLANE_URL` in `index.html`:

```javascript
const CONTROL_PLANE_URL = 'http://your-control-plane:9090';
```

## Design

- **Modern UI**: Gradient backgrounds, smooth animations
- **Responsive**: Works on desktop and mobile
- **Real-time**: WebSocket-ready architecture
- **Accessible**: WCAG 2.1 AA compliant

## Technology Stack

- Pure HTML/CSS/JavaScript
- No build process required
- Fetch API for REST calls
- CSS Grid & Flexbox layouts

## Customization

### Colors
Edit the CSS gradients in `<style>` section:

```css
background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
```

### Refresh Rate
Change update interval (default 2s):

```javascript
setInterval(updateDashboard, 2000); // milliseconds
```

## Browser Support

- Chrome 90+
- Firefox 88+
- Safari 14+
- Edge 90+
