import os
from pathlib import Path

from django.contrib.messages import constants as messages


# Build paths inside the project like this: BASE_DIR / 'subdir'.
BASE_DIR = Path(__file__).resolve().parent.parent.parent


# Models
# https://docs.djangoproject.com/en/3.0/ref/settings/#models

INSTALLED_APPS = [
    'django.contrib.admin',
    'django.contrib.auth',
    'django.contrib.contenttypes',
    'django.contrib.sessions',
    'django.contrib.messages',
    'django.contrib.staticfiles',
    'django.contrib.sitemaps',

    'wagtail_modeladmin',
    "wagtail.contrib.routable_page",
    'wagtail.contrib.redirects',
    'wagtail.embeds',
    'wagtail.sites',
    'wagtail.users',
    'wagtail.snippets',
    'wagtail.documents',
    'wagtail.images',
    'wagtail.search',
    'wagtail.admin',
    'wagtail',

    'taggit',
    'modelcluster',

    'admin',
    'accounts',
    'pages',
    'scheduler',
    'mail',
]

MIDDLEWARE = [
    'django.middleware.security.SecurityMiddleware',

    'wagtail.contrib.redirects.middleware.RedirectMiddleware',
    'whitenoise.middleware.WhiteNoiseMiddleware',

    'django.contrib.sessions.middleware.SessionMiddleware',
    'django.middleware.common.CommonMiddleware',
    'django.middleware.csrf.CsrfViewMiddleware',
    'django.contrib.auth.middleware.AuthenticationMiddleware',
    'django.contrib.messages.middleware.MessageMiddleware',
    'django.middleware.clickjacking.XFrameOptionsMiddleware',
]

ROOT_URLCONF = 'blog.urls'

TEMPLATES = [
    {
        'BACKEND': 'django.template.backends.django.DjangoTemplates',
        'DIRS': [BASE_DIR / 'blog/templates'],
        'APP_DIRS': True,
        'OPTIONS': {
            'context_processors': [
                'django.template.context_processors.debug',
                'django.template.context_processors.request',
                'django.contrib.auth.context_processors.auth',
                'django.contrib.messages.context_processors.messages',

                'blog.context_processors.canonical',
                'blog.context_processors.base_url',
                'blog.context_processors.nav_items',
                'blog.context_processors.site_owner',
                'blog.context_processors.newsletter_page',
            ],
        },
    },
]

WSGI_APPLICATION = 'blog.wsgi.application'


# Messages
# https://docs.djangoproject.com/en/3.0/ref/settings/#messages

MESSAGE_TAGS = {
    messages.DEBUG: "alert-info",
    messages.INFO: "alert-info",
    messages.SUCCESS: "alert-success",
    messages.WARNING: "alert-warning",
    messages.ERROR: "alert-danger",
}


# Internationalization
# https://docs.djangoproject.com/en/4.0/topics/i18n/

LANGUAGE_CODE = 'en-us'
TIME_ZONE = 'UTC'
USE_I18N = True
USE_TZ = True
USE_THOUSAND_SEPARATOR = False  # NOTE: If setting to true this could cause issues with some srcset tags.


# Static files (CSS, JavaScript, Images)
# https://docs.djangoproject.com/en/4.0/howto/static-files/

STATIC_URL = 'static/'
STORAGES = {
    "default": {
        "BACKEND": "django.core.files.storage.FileSystemStorage",
    },
    "staticfiles": {
        "BACKEND": "whitenoise.storage.CompressedManifestStaticFilesStorage",
    },
}
STATICFILES_DIRS = (BASE_DIR / "blog/static",)
STATIC_ROOT = BASE_DIR / "static"


# Media files (Images, Videos)
# https://docs.djangoproject.com/en/4.0/ref/settings/#media-root

MEDIA_URL = 'media/'
MEDIA_ROOT = BASE_DIR / "media"


# Default primary key field type
# https://docs.djangoproject.com/en/4.0/ref/settings/#default-auto-field

DEFAULT_AUTO_FIELD = 'django.db.models.BigAutoField'


# Auth
# https://docs.djangoproject.com/en/3.0/ref/settings/#auth

AUTH_USER_MODEL = "accounts.User"


# Wagtail
# https://docs.wagtail.org/en/stable/reference/settings.html#settings

WAGTAIL_SITE_NAME = "Blog"
WAGTAIL_MODERATION_ENABLED = False
WAGTAILADMIN_COMMENTS_ENABLED = False
WAGTAIL_FRONTEND_LOGIN_URL = '/accounts/login/'
TAG_SPACES_ALLOWED = False
WAGTAIL_AUTO_UPDATE_PREVIEW = True
WAGTAIL_USAGE_COUNT_ENABLED = True
WAGTAILADMIN_RICH_TEXT_EDITORS = {
    'default': {
        'WIDGET': 'wagtail.admin.rich_text.DraftailRichTextArea',
        'OPTIONS': {
            'features': ['h2', 'h3', 'bold', 'italic', 'link', 'document-link','ol', 'ul', 'hr', 'blockquote', 'strikethrough', 'code']
        }
    }
}
WAGTAIL_WORKFLOW_ENABLED = False


# django-taggit
# https://django-taggit.readthedocs.io/en/latest/getting_started.html?highlight=TAGGIT_CASE_INSENSITIVE#getting-started

TAGGIT_CASE_INSENSITIVE = True
