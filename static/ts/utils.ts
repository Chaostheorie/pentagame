import Toast from 'bootstrap/js/dist/toast';
import Modal from 'bootstrap/js/dist/modal';
import Dropdown from 'bootstrap/js/dist/dropdown';
import Collapse from 'bootstrap/js/dist/collapse';
import Alert from 'bootstrap/js/dist/alert';
import Tooltip from 'bootstrap/js/dist/tooltip';
import { Request } from './interfaces/requests';
import { sanitize } from 'dompurify';

// function to create a simple 8 letter to string for ids as appendix when creating multiple elements
function random_id(): String {
    return Math.random().toString().substr(2, 8);
}

// handy bind to build HTMLElements
// NOTE: I'm pretty sure this is not as fast as possible but it's making code a lot more tidy
export function build_element(
    element: string,
    classes?: string[],
    attributes?: { [key: string]: string },
    innerHTML?: string
): HTMLElement {
    let new_element = document.createElement(element);

    if (attributes !== null && attributes !== undefined) {
        Object.keys(attributes).forEach((key) =>
            new_element.setAttribute(key, attributes[key])
        );
    }

    if (classes !== null && classes !== undefined) {
        classes.forEach((style_class) =>
            new_element.classList.add(style_class)
        );
    }

    if (innerHTML !== undefined && innerHTML !== null) {
        new_element.innerHTML = innerHTML;
    }

    return new_element;
}

// function to create an alert
export type LevelType = 0 | 1 | 2;
export function create_alert(
    level: LevelType,
    title: string,
    message: string,
    show?: boolean, // default: true
    autohide?: boolean // default: true
): String {
    // toast id
    let id = `toast-${random_id()}`;

    // evaluate level
    let icon, level_class;
    switch (level) {
        case 0:
            icon = 'exclamation-octagon-fill';
            level_class = 'danger';
            break;
        case 2:
            icon = 'exclamation-triangle-fill';
            level_class = 'warning';
            break;
        case 1:
            icon = 'info-circle-fill';
            level_class = 'info';
            break;
        default:
            icon = 'exclamation-octagon-fill';
            level_class = 'danger';
            break;
    }

    // create new toast
    let toast = build_element('div', ['toast', 'mt-4', `bg-${level_class}`], {
        id: id,
        role: 'alert',
        'aria-atomic': 'true',
        'aria-live': 'assertive',
    });

    // add subheader

    // create header
    let header = build_element('div', [
        'toast-header',
        `bg-${level}`,
        'h6',
        'text-dark',
    ]);
    let brand = build_element('i', ['me-2', 'bi', `bi-${icon}`], {
        alt: `alert ${level} icon`,
    });

    let toast_title = build_element('strong', ['me-auto'], {}, title);

    let close_button = build_element('button', ['btn-close'], {
        type: 'button',
        'data-dismiss': 'toast',
        'aria-label': 'Close',
    });

    // glue header
    header.appendChild(brand);
    header.appendChild(toast_title);
    header.appendChild(close_button);

    // add message
    let body = build_element('div', ['toast-body', 'text-white'], {}, message);

    // glue toast and attach to toast container
    toast.appendChild(header);
    toast.appendChild(body);
    document.getElementById('toast-container').appendChild(toast);
    let el = new Toast(toast, { autohide: autohide !== false ? true : false });

    // bind close button (in case bootstrap acts up)
    close_button.addEventListener('click', (_: MouseEvent) => {
        el.dispose();
    });

    // evaluate optional arguments
    if (show !== false) {
        el.show();
    }

    // return id to allow access by calling function
    return id;
}

export function create_modal(
    text: string,
    title: string,
    callback: (event: Event) => void,
    level?: LevelType, // default: 0
    confirmation?: string
): String {
    // generate id
    let id = `modal-${random_id()}`,
        label_id = `${id}-label`;

    // evaluate level
    let icon, level_class;
    switch (level) {
        case 0:
            icon = 'exclamation-octagon-fill';
            level_class = 'danger';
            break;
        case 2:
            icon = 'exclamation-triangle-fill';
            level_class = 'warning';
            break;
        case 1:
            icon = 'info-circle-fill';
            level_class = 'info';
            break;
        default:
            icon = 'exclamation-octagon-fill';
            level_class = 'danger';
            break;
    }

    // create modal
    let modal = build_element('div', ['modal', 'fade'], {
        id: id,
        tabindex: '-1',
        'aria-labelledby': label_id,
        'aria-hidden': 'true',
    });

    // create inner modal dialog and content
    let modal_dialog = build_element('div', ['modal-dialog']);
    let modal_content = build_element('div', [
        'modal-content',
        'text-white',
        `bg-${level_class}`,
    ]);

    // build modal header
    let modal_header = build_element('div', ['modal-header']),
        modal_close_btn = build_element(
            'button',
            ['btn-close', 'btn-close-white'],
            {
                'data-bd-dismiss': 'modal',
                'aria-label': 'Close',
            }
        ),
        modal_label = build_element(
            'h5',
            ['modal-title'],
            {
                id: label_id,
            },
            `<i class="bi bi-${icon}"></i> ${title}`
        );

    // glue header elements
    modal_header.appendChild(modal_label);
    modal_header.appendChild(modal_close_btn);
    modal_content.appendChild(modal_header);

    // create modal body
    let modal_body = build_element('div', ['modal-body'], {}, text);
    modal_content.appendChild(modal_body);

    // create modal footer
    let modal_footer = document.createElement('div');
    modal_footer.classList.add('modal-footer');

    // create buttons for footer
    let dismiss_button = build_element(
            'button',
            ['btn', 'btn-outline-light'],
            {
                type: 'button',
                'data-bs-dismiss': 'modal',
                'aria-label': 'Close',
            },
            'Cancel'
        ),
        confirm_button = build_element(
            'button',
            ['btn', 'btn-outline-light'],
            {
                type: 'button',
            },
            confirmation === undefined ? 'Confirm' : confirmation
        );

    confirm_button.addEventListener('click', callback);

    // attach buttons to footer and footer to content
    modal_footer.appendChild(dismiss_button);
    modal_footer.appendChild(confirm_button);
    modal_content.appendChild(modal_footer);

    // attach content to dialog, dialog to modal and modal to body
    modal_dialog.appendChild(modal_content);
    modal.appendChild(modal_dialog);
    document.body.appendChild(modal);
    let bs_modal = new Modal(modal, {
        keyboard: false,
        backdrop: 'static',
        focus: true,
    });
    bs_modal.show();

    // attach event listener to close button
    modal_close_btn.addEventListener('click', (event) => {
        event.preventDefault();

        // bs_modal.dispose(); - This is unfortunately buggy in 5.0.0-beta1

        // manually destroy elements
        document.querySelectorAll('.modal-backdrop').forEach((element) => {
            element.remove();
        });

        modal.remove();
    });

    return id;
}

// send wither POST or GET requests to the server and get a result synchronously
export function getJSONP(url: string, method?: string, body?: Request): any {
    let xhr = new XMLHttpRequest();

    // open connection
    if (method === 'POST') {
        xhr.open('POST', url);
    } else {
        xhr.open('GET', url);
    }

    xhr.onload = () => {
        if (xhr.status >= 200 && xhr.status < 300) {
            try {
                return JSON.parse(xhr.response);
            } catch (e) {
                console.error('[UTILS]: Response from server was not parsable');
                console.debug(xhr.response);
            }
        } else if (xhr.status == 401) {
            console.log('[UTILS]: Unauthorized request was gracefully caught');
            return null;
        } else {
            console.error('[UTILS]: Response from server seems to be an error');
            console.debug(xhr.statusText);
            throw Error('Response with error code was returned');
        }
    };

    xhr.onerror = () => xhr.statusText;
    if (body === undefined) {
        xhr.send();
    } else {
        xhr.send(body.as_string());
    }

    return xhr.response;
}

// thread blocking sleep
export function sleep(milliseconds: number): void {
    const date = Date.now();
    let currentDate = null;
    do {
        currentDate = Date.now();
    } while (currentDate - date < milliseconds);
}

// trigger download for any utf 8 encoded data
// I'm seriously wondering why it's this easy to trigger a download
export function download(filename: string, text: string) {
    var element = build_element('a', [], {
        href: 'data:text/plain;charset=utf-8,' + encodeURIComponent(text),
        download: filename,
    });

    element.style.display = 'none';
    document.body.appendChild(element);

    element.click();

    document.body.removeChild(element);
}

export function on_load(callback: Function) {
    if (document.readyState !== 'loading') {
        callback();
    } else {
        document.addEventListener('DOMContentLoaded', function () {
            callback();
        });
    }
}

export function redirect(url: string, method?: 'POST' | 'GET') {
    if (method === undefined || method === 'GET') {
        window.location.href = url;
    } else {
        let form = document.createElement('form');
        document.body.appendChild(form);
        form.method = 'post';
        form.action = url;
        form.submit();
    }
}

export function init_ui(context: string) {
    document.querySelectorAll('.toast').forEach((toastEl: HTMLElement) => {
        return new Toast(toastEl, { autohide: false });
    });

    document
        .querySelectorAll('[data-bs-toggle="tooltip"]')
        .forEach((tooltipTriggerEl: HTMLElement) => {
            return new Tooltip(tooltipTriggerEl);
        });

    document.querySelectorAll('.alert').forEach((alert: HTMLElement) => {
        new Alert(alert);
    });

    document.querySelectorAll('.btn-close').forEach((btn: HTMLElement) => {
        let target = btn.getAttribute('data-target');
        if (target !== null) {
            btn.addEventListener('click', (_: Event) => {
                document.querySelector(target).remove();
            });
        }
    });

    /*
    document
        .querySelectorAll('.dropdown-toggle')
        .forEach((dropdownToggleEl: HTMLElement) => {
            return new Dropdown(dropdownToggleEl, {
                display: 'dynamic',
            });
        });*/

    let auth_status_el = document.getElementById('auth-status');
    if (auth_status_el.getAttribute('value') === 'true') {
        try {
            let alerts: [LevelType, string][] = getJSONP(
                '/api/users/alerts',
                'GET'
            );
            if (alerts === null || alerts.length == 0) {
                console.info(`[${context}]: No notifications available. 😼`);
            } else {
                console.info(`[${context}]: Showing alerts. 🐱`);
                for (let i = alerts.length - 1; i > -1; i--) {
                    create_alert(
                        alerts[i][0],
                        sanitize(alerts[i][1]),
                        'Notification'
                    );
                }
            }
        } catch (e) {
            console.log(
                `[${context}]: Failed to retrieve alerts. It's possible that authentication cookies are outdated`
            );
            console.log(e);
        }
    }

    let el = document.getElementById('hidden-alert');
    if (el !== null && el !== undefined) {
        create_alert(1, 'Alert', el.innerHTML);
    }

    console.info(`[${context}]: Finished Initializing general UI Elements 🐱`);
}
