# WormholeSystems - EVE Online Wormhole Mapping Tool

A comprehensive wormhole mapping and tracking application for EVE Online, built with Laravel 12, Inertia.js, Vue 3, and
Tailwind CSS.

## Requirements

- PHP 8.4 or higher
- Composer
- MySQL or MariaDB
- Node.js and NPM (for the frontend)
- Redis (for caching and queues)
- [Laravel Herd](https://herd.laravel.com/) (recommended)

**We strongly recommend using Laravel Herd** as it provides all necessary services pre-configured, including PHP, MySQL,
Redis, and automatic HTTPS setup.

## Installation

### 1. Clone and Setup Project

```bash
git clone https://github.com/WormholeSystems/WormholeSystems.git
cd WormholeSystems
```

### 2. Laravel Herd Setup

If you haven't already, install [Laravel Herd](https://herd.laravel.com/) and ensure the following services are running:

- **MySQL** (port 3306)
- **Redis** (port 6379)
- **Reverb** (Broadcasting service - must be enabled in Herd's Services panel)

You can check service status and start them using Herd's interface. **Important**: You must enable Reverb under the "
Broadcasting" section in Herd's Services panel for real-time features to work.

### 3. Environment Configuration

Create your environment file:

```bash
cp .env.example .env
```

Update the following key configuration values in your `.env` file:

```env
# Application
APP_NAME=WormholeSystems
APP_URL=https://tunnelvision.test
APP_DEBUG=true

# Database (Herd defaults)
DB_CONNECTION=mysql
DB_HOST=127.0.0.1
DB_PORT=3306
DB_DATABASE=wormholesystems
DB_USERNAME=root
DB_PASSWORD=

# Cache & Queue (using Redis)
CACHE_STORE=redis
QUEUE_CONNECTION=database

# Redis (Herd defaults)
REDIS_HOST=127.0.0.1
REDIS_PORT=6379

# EVE Online SSO (see EVE App Setup section)
EVE_CLIENT_ID=your_eve_client_id
EVE_CLIENT_SECRET=your_eve_client_secret

# Broadcasting (Laravel Reverb)
BROADCAST_CONNECTION=reverb
REVERB_APP_ID=1001
REVERB_APP_KEY=laravel-herd
REVERB_APP_SECRET=secret
REVERB_HOST="reverb.herd.test"
REVERB_PORT=443
REVERB_SCHEME=https
```

### 4. Install Dependencies

Install PHP dependencies:

```bash
composer install
```

Install frontend dependencies:

```bash
npm install
```

Generate application key:

```bash
php artisan key:generate
```

### 5. Database Setup

Create the database (if using Herd's MySQL) named `wormholesystems` or update `.env` with your database name.

Run migrations:

```bash
php artisan migrate
```

### 6. EVE Online SDE (Static Data Export) Setup

Download the EVE Online SDE data:

```bash
php artisan sde:download
```

Prepare the SDE (this may require increased memory limits):

```bash
php artisan sde:prepare
```

Seed the database with SDE data:

```bash
php artisan db:seed
```

### 7. Build Frontend Assets

For development:

```bash
npm run dev
```

For production:

```bash
npm run build
```

### 8. Laravel Reverb Setup (Real-time Features)

**Enable Reverb in Laravel Herd:**

1. **Open Laravel Herd** and go to the "Services" tab
2. **Find "Broadcasting"** section and click "Add Service"
3. **Select "Reverb"** from the available broadcasting services
4. **Start the service** - it will run on port 8080 and be accessible via `reverb.herd.test:443`

**Configure your application:**

5. **Environment variables** are already configured for Herd in the .env section above
6. **Test WebSocket connection** by checking browser console for connection to `wss://reverb.herd.test:443`

**Note**: Reverb must be manually enabled in Herd's Services panel under "Broadcasting" - it's not enabled by default.

## EVE Online Application Setup

To enable EVE Online SSO authentication, you need to create an application on the EVE Developers portal:

### 1. Create EVE Application

1. Go to [EVE Developers](https://developers.eveonline.com/)
2. Sign in with your EVE Online account
3. Navigate to "Applications" and click "Create New Application"
4. Fill out the application details:
    - **Application Name**: WormholeSystems (or your preferred name)
    - **Description**: Wormhole mapping and tracking tool
    - **Connection Type**: Authentication & API Access
    - **Permissions**: Select the required scopes (see below)
    - **Callback URL**: `https://tunnelvision.test/auth/eve/callback` (adjust domain as needed)

### 2. Required EVE Online Scopes

The application requires the following ESI scopes:

- `esi-location.read_location.v1` - Read character location (solar system, station, structure)
- `esi-location.read_online.v1` - Read character online status
- `esi-location.read_ship_type.v1` - Read current ship information (ship type, name)
- `esi-ui.write_waypoint.v1` - Set autopilot waypoints in-game

**Note**: The application will function with basic authentication without scopes, but the above scopes are required for
full functionality including character tracking, location updates, and autopilot features.

### 3. Configure Application

After creating the application, copy the **Client ID** and **Client Secret** to your `.env` file:

```env
EVE_CLIENT_ID=your_client_id_here
EVE_CLIENT_SECRET=your_client_secret_here
```

## Running the Application

### Development Mode

1. **Start the development server** (Herd automatically serves your site at `https://tunnelvision.test`)

2. **Start the frontend build process**:
   ```bash
   npm run dev
   ```

3. **Start the queue worker** (for background jobs):
   ```bash
   php artisan queue:work
   ```

4. **Start the scheduler** (for cron-like tasks):
   ```bash
   php artisan schedule:work
   ```

5. **Laravel Reverb** (for real-time features):

   **Important**: Ensure Reverb is enabled in Herd's Services panel under "Broadcasting". It's not enabled by default
   and must be manually added and started.

### Production Setup

For production deployment:

1. **Build assets**:
   ```bash
   npm run build
   ```

2. **Optimize Laravel**:
   ```bash
   php artisan optimize
   php artisan config:cache
   php artisan route:cache
   php artisan view:cache
   ```

3. **Setup supervisor for queue workers** (example configuration):
   ```ini
   [program:wormholesystems-worker]
   process_name=%(program_name)s_%(process_num)02d
   command=php /path/to/tunnelvision/artisan queue:work --sleep=3 --tries=3 --max-time=3600
   autostart=true
   autorestart=true
   stopasgroup=true
   killasgroup=true
   user=www-data
   numprocs=2
   redirect_stderr=true
   stdout_logfile=/var/log/wormholesystems-worker.log
   stopwaitsecs=3600
   ```

4. **Setup cron for scheduler**:
   ```bash
   * * * * * cd /path/to/tunnelvision && php artisan schedule:run >> /dev/null 2>&1
   ```

## Background Services

### Queue System

The application uses Laravel's queue system for background processing. Key jobs include:

- Fetching killmail data from zKillboard
- Updating character locations and online status
- Processing sovereignty data

**Queue Commands:**

- `php artisan queue:work` - Start processing jobs
- `php artisan queue:listen` - Start queue listener
- `php artisan queue:failed` - View failed jobs
- `php artisan queue:retry all` - Retry all failed jobs

### Scheduled Tasks

The following tasks run automatically via Laravel's scheduler:

- **Every minute**: Server status updates, character online status
- **Every 10 seconds**: Character location updates (when server online)
- **Every 10 minutes**: Connection age checks, old signature cleanup
- **Daily at 15:00**: Sovereignty data updates
- **Weekly**: Killmail data for last 90 days

**Scheduler Commands:**

- `php artisan schedule:list` - View all scheduled tasks
- `php artisan schedule:run` - Run scheduled tasks once
- `php artisan schedule:work` - Start the schedule worker

### Real-time Features

Laravel Reverb provides WebSocket support for real-time updates. With Laravel Herd:

- **Must be enabled manually** - Add "Reverb" service under "Broadcasting" in Herd's Services panel
- **Access via**: `reverb.herd.test:443` (HTTPS)
- **Managed through Herd interface** - Start/stop/restart via Herd's Services panel
- **Environment variables** - Pre-configured for Herd (see .env section above)

**Manual Commands** (if not using Herd):

- `php artisan reverb:start` - Start the Reverb server manually
- `php artisan reverb:restart` - Restart the Reverb server

## Useful Commands

### Application Management

```bash
# Clear all caches
php artisan optimize:clear

# View application information
php artisan about

# Access tinker (interactive PHP shell)
php artisan tinker
```

### Database Management

```bash
# Check migration status
php artisan migrate:status

# Fresh migration (drops all tables)
php artisan migrate:fresh --seed

# Show database information
php artisan db:show
```

### EVE Online Data

```bash
# Download and seed SDE data
php artisan sde:download
php artisan sde:prepare
php artisan db:seed

# Get killmails for specific day
php artisan app:get-killmails-for-day 2024-01-15

# Listen for live killmails
php artisan app:listen-for-killmails
```

### Development Tools

```bash
# Run tests
php artisan test

# Format code with Pint
vendor/bin/pint

# Generate API documentation
php artisan scribe:generate
```

## Troubleshooting

### Common Issues

1. **Memory limit errors during SDE seeding**:
    - Increase PHP memory limit: `php -d memory_limit=2G artisan sde:seed`

2. **Queue jobs not processing**:
    - Ensure Redis is running
    - Check queue worker is active: `php artisan queue:work`

3. **Real-time updates not working**:
    - Verify Reverb server is running: `php artisan reverb:start`
    - Check WebSocket connection in browser console

4. **EVE SSO authentication failing**:
    - Verify EVE_CLIENT_ID and EVE_CLIENT_SECRET are correct
    - Ensure callback URL matches EVE application settings
    - Check required scopes are granted

### Log Files

- Application logs: `storage/logs/laravel.log`
- Queue logs: Check supervisor logs if using production setup
- Web server logs: Check Herd logs or your web server logs

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Run tests: `php artisan test`
5. Format code: `vendor/bin/pint`
6. Submit a pull request

## License

This project is open-sourced software licensed under the [MIT license](LICENSE).
