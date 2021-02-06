import { getJSONP, init_ui, on_load, build_element } from './utils';
import { sanitize } from 'dompurify';

on_load(() => {
    // init ui
    init_ui('OVERVIEW');

    console.log('[OVERVIEW]: Starting to fetch latest games');

    // load overview
    let auth_status = <HTMLInputElement>document.getElementById('auth-status');

    // remove placeholder
    let container = document.getElementById('games-container');
    container.innerHTML = '';

    console.log('[OVERVIEW]: Fetching latest games');
    let games = getJSONP('/api/games/latest'); // This is not guarded ATM

    console.log(`[OVERVIEW]: Fetched a total of ${games.length}`);

    if (games.length > 0) {
        // create join form
        let join_form = build_element('form', ['w-80']),
            join_input_group = build_element('div', ['input-group', 'mb-2']),
            join_target_field = build_element('input', ['form-control'], {
                placeholder: 'GAME ID',
                'aria-label': 'Game ID for joining a game directly',
                'aria-describedby': 'join-game-label',
                id: 'game-id',
                name: 'game-id',
                type: 'text',
            }),
            join_form_submit_button = build_element(
                'button',
                ['btn', 'btn-outline-dark'],
                {
                    type: 'submit',
                },
                'Join Game'
            );

        // style form and form elements
        join_form.addEventListener('submit', (event) => {
            event.preventDefault();
            let id = (<HTMLInputElement>document.getElementById('game-id'))
                .value;
            window.location.href = `/games/join/${id}`;
        });
        // glue form together and attach to container
        join_input_group.appendChild(join_target_field);
        join_input_group.appendChild(join_form_submit_button);
        join_form.appendChild(join_input_group);
        container.appendChild(join_input_group);

        // create game list
        let game_list = document.createElement('ul');
        game_list.classList.add(
            'list-group',
            'border-1',
            'border-dark',
            'list-group-flush'
        );

        for (let i = 0; i < games.length; i++) {
            let fragment = games[i],
                game = build_element(
                    'a',
                    ['list-group-item', 'dark-link'],
                    {
                        href: `/games/join/${fragment[0]}`,
                    },

                    `<i class="bi ${sanitize(fragment[2])}"></i>
      ${sanitize(fragment[1]).trim()}
      <span class="text-darker">#${fragment[0]}</span>
      `
                );

            games.appendChild(game);
        }

        if (games.length > 5) {
            // build and attach "see more" link
            let see_more = build_element(
                'a',
                ['list-group-item', 'dark-link'],
                { href: '/games/browse/1' },
                'See more <br /> <i class="bi bi-arrow-down-square-fill"></i>'
            );

            games.appendChild(see_more);
        }

        container.appendChild(games);
    } else {
        // create placeholder elements
        let link = build_element(
            'a',
            ['btn', 'btn-block', 'btn-outline-dark'],
            {
                href: '/games/create',
            },
            'Be the first to create a game'
        );

        // append link to container
        container.appendChild(link);
    }
    console.log('[OVERVIEW]: Done page-specific initalizing UI');
});
