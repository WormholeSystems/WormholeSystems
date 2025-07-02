
## Requirements

In order to get started with the project you will need to have the following installed:

- PHP 8.4 or higher
- Composer
- MySQL or MariaDB
- NodeJS and NPM (for the frontend)

I recommend installing [Laravel Herd](https://herd.laravel.com/) for a smooth development experience, as it comes with all the necessary tools pre-configured.

## Installation

After cloning the repository, copy the `.env.example` file to `.env` and update the database connection settings to match your local environment. 

Once you have your `.env` file set up, run the following command to install the project dependencies:

```bash
composer install
```

After that, you will need to install the frontend dependencies. You can do this by running:

```bash
npm install
```

Also make sure to set the `APP_KEY` by running 

```bash
php artisan key:generate
```

Next, configure the `APP_URL` in your `.env` file to match your local development URL, for example: 

```env
APP_URL=https://tunnelvision.test
```

Laravel Herd makes setting up SSL and domains easy, so you can use a custom domain like `tunnelvision.test` for local development.

### Seeding the Database

To seed the database with the EVE Online SDE (Static Data Export), you will need to download the SDE files. You can do this by running the following command:

```bash
php artisan sde:download
```

Prepare the SDE by running:

```bash
php artisan sde:prepare
```

Note that this might use a lot of memory, so you might need to increase the memory limit in your `php.ini` file.

Finally, run the migration files to set up the database:

```bash
php artisan migrate
```

and seed the database with the SDE data:

```bash
php artisan sde:seed
```
