import { sanitize } from 'dompurify';
import { on_load, create_modal, redirect } from './utils';

on_load(() => {
    // set value of status
    let status = document.getElementById('status-value').getAttribute('value'),
        status_el = document.getElementById('status');
    status_el.innerHTML = sanitize(status);

    // bind 'remove account' btn
    document
        .getElementById('archive-account')
        .addEventListener('click', (event) => {
            event.preventDefault();
            create_modal(
                "This will make your account be shown as archived. You will not be able to log into this account anymore but you're game history will be kept.",
                'Archiving your Account',
                () => {
                    console.info(
                        '[Settings]: Sending request for account archiving. 😿 Thank you for using our service.'
                    );

                    // send the data
                    redirect('/users/settings/archive', 'POST');
                },
                1,
                'Continue'
            );
        });

    // bind 'delete account' btn
    document
        .getElementById('delete-account')
        .addEventListener('click', (event) => {
            event.preventDefault();
            create_modal(
                "This will delete all data related to your account including your game history. THIS IS NOT REVERSIBLE. We won't be able to restore your account.",
                'Deleting your Account',
                () => {
                    console.info(
                        '[Settings]: Sending request for account removal. 😿 Thank you for using our service '
                    );

                    // send the data
                    redirect('/users/settings/delete', 'POST');
                },
                0,
                'Confirm'
            );
        });

    console.info('[Settings]: Finished Initializing Settings UI 🐱');
});
