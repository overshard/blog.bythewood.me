from django.templatetags.static import static
from django.utils.html import escape, format_html
from wagtail_modeladmin.mixins import ThumbnailMixin
from wagtail_modeladmin.options import ModelAdmin, modeladmin_register
from wagtail import hooks
from wagtail.rich_text import LinkHandler

from pages.models import BlogPostPage
from scheduler.models import ScheduledTask
from mail.models import Subscriber


# @hooks.register("insert_global_admin_js", order=100)
# def global_admin_js():
#     return format_html('<script src="{}"></script>', static("admin.global.js"))


@hooks.register("insert_global_admin_css", order=100)
def global_admin_css():
    return format_html('<link rel="stylesheet" href="{}">', static("admin.global.css"))


@hooks.register("insert_editor_js", order=100)
def editor_js():
    return format_html('<script src="{}"></script>', static("admin.editor.js"))


@hooks.register("insert_editor_css", order=100)
def editor_css():
    return format_html('<link rel="stylesheet" href="{}">', static("admin.editor.css"))


class ExternalLinkHandler(LinkHandler):
    identifier = 'external'

    @classmethod
    def expand_db_attributes(cls, attrs):
        href = attrs["href"]
        return '<a href="%s" target="_blank" rel="noopener noreferrer">' % escape(href)


@hooks.register('register_rich_text_features')
def register_external_link_handler(features):
    features.register_link_type(ExternalLinkHandler)


class BlogPostPageAdmin(ThumbnailMixin, ModelAdmin):
    model = BlogPostPage
    menu_label = 'Posts'
    menu_icon = 'doc-full-inverse'
    exclude_from_explorer = True
    menu_order = 200
    list_display = ('admin_thumb', 'title', 'first_published_at', 'tags_list', 'live')
    list_filter = ('tags', 'live')
    search_fields = ('title', 'body')
    ordering = ('-first_published_at',)
    thumb_image_field_name = 'cover_image'
    thumb_image_width = 75

    def get_queryset(self, request):
        return super().get_queryset(request).prefetch_related('tags')

    def tags_list(self, obj):
        return ', '.join(o.name for o in obj.tags.all())


modeladmin_register(BlogPostPageAdmin)


class ScheduledTaskAdmin(ModelAdmin):
    model = ScheduledTask
    menu_label = 'Scheduler'
    menu_icon = 'time'
    add_to_settings_menu = True
    menu_order = 900
    list_display = ('management_command', 'run_interval', 'last_run_at', 'next_run_at')
    list_filter = ('run_interval',)
    search_fields = ('management_command',)
    ordering = ('-run_interval',)


modeladmin_register(ScheduledTaskAdmin)


class SubscriberAdmin(ModelAdmin):
    model = Subscriber
    menu_label = 'Subscribers'
    menu_icon = 'mail'
    add_to_settings_menu = True
    menu_order = 1000
    list_display = ('email', 'created_at')
    search_fields = ('email',)
    ordering = ('-created_at',)


modeladmin_register(SubscriberAdmin)
