// What a teammate does during a quiz, end to end against the real engine. Expected values are official FS-Quiz
// answers (question id in the test name) unless stated.
import { answer, choice, copied, expect, field, openFromPalette, pasteAnywhere, switchBox, test } from './fixtures';

const SKIDPAD =
    'A Formula Student car of mass 240 kg with a lift coefficient times area of 3.2 m² drives the skidpad ' +
    '(centreline radius 9.125 m). With a tyre friction coefficient of 1.4 and air density 1.1 kg/m³, what is the ' +
    'maximum speed?';

test('Q90: paste a problem anywhere, open the top match pre-filled, copy the answer', async ({ app }) => {
    await pasteAnywhere(app, SKIDPAD);
    await expect(app.getByLabel('Values found in the problem')).toContainText('3.2 m²');
    const top = app.getByLabel('Scripts that fit').getByRole('button').first();
    await expect(top).toContainText('Max cornering speed with downforce');
    await expect(top).toContainText('5');
    await top.click();
    await expect(field(app, 'mu')).toHaveValue('1.4');
    await expect(field(app, 'm')).toHaveValue('240 kg');
    // pre-filled fields say where the value came from, and every detected value is used by this sheet
    await expect(app.locator('label.field[data-var="mu"] .from')).toContainText('from the problem');
    await expect(app.getByLabel('Values found in the problem').getByText('not used')).toHaveCount(0);
    await field(app, 'mu').fill('1.5');
    await expect(app.locator('label.field[data-var="mu"] .from')).toHaveCount(0);
    await field(app, 'mu').fill('1.4');
    await expect(answer(app)).toHaveText(/^11\.759\d* m\/s$/);
    await app.getByRole('button', { name: /Copy 11\.76/ }).click();
    expect(await copied(app)).toEqual(['11.76']);
});

test('a question that asks for km/h and one decimal opens in that format, and Use my Settings instead undoes it', async ({ app }) => {
    await pasteAnywhere(app, `${SKIDPAD} Give your answer in km/h to 1 decimal place.`);
    await app.getByLabel('Scripts that fit').getByRole('button').first().click();
    await expect(answer(app)).toHaveText(/^42\.33\d* km\/h$/);
    await expect(app.locator('.a-unit input')).toHaveValue('km/h');
    await expect(app.locator('.a-format')).toContainText('Rounded to 1 decimal and shown in km/h, as the question asks');
    await app.getByRole('button', { name: /Copy 42\.3/ }).click();
    expect(await copied(app)).toEqual(['42.3']);
    await app.getByRole('button', { name: 'Use my Settings instead' }).click();
    await expect(answer(app)).toHaveText(/^11\.759\d* m\/s$/);
    await expect(app.getByRole('button', { name: /Copy 11\.76/ })).toBeVisible();
});

test('rehearsal: label the question, copy, and find the entry in the session log; the help key opens Help', async ({ app }) => {
    await pasteAnywhere(app, SKIDPAD);
    await app.getByLabel('Scripts that fit').getByRole('button').first().click();
    await expect(answer(app)).toHaveText(/^11\.759\d* m\/s$/);
    await app.getByLabel('Question #').fill('Q90');
    await app.getByRole('button', { name: /Copy 11\.76/ }).click();
    expect(await copied(app)).toEqual(['11.76']);

    await app.getByRole('button', { name: 'Session log', exact: true }).click();
    await expect(app.getByRole('heading', { level: 1, name: 'Session log', exact: true })).toBeVisible();
    const row = app.locator('tbody tr').first();
    await expect(row).toContainText('Q90');
    await expect(row).toContainText('Max cornering speed with downforce');
    await expect(row).toContainText('11.76');
    await expect(row).toContainText('mu=1.4');

    await app.keyboard.press('ControlOrMeta+/');
    await expect(app.getByRole('heading', { level: 1, name: 'Help' })).toBeVisible();
    await app.keyboard.press('ControlOrMeta+/');
    await expect(app.getByRole('heading', { level: 1, name: 'Session log', exact: true })).toBeVisible();
});

test('Q245: palette search, type the knowns, pick the root that answers the question', async ({ app }) => {
    await openFromPalette(app, 'discharge', /TS discharge/);
    await field(app, 'V_0').fill('396 V');
    await field(app, 'V_c').fill('60 V');
    await field(app, 't').fill('5 s');
    await field(app, 'C').fill('1800 uF');
    const peak = app.locator('.results .res[data-var="P_pk"]');
    await expect(peak).toContainText('106.53');
    await peak.click();
    await expect(answer(app)).toHaveText(/^106\.53\d* W$/);
    // the discharge resistor itself is also solved
    await expect(app.locator('.results .res[data-var="R"]')).toContainText('1472');
});

test('Q34: a worked example loads its inputs and solves, with the second root explained', async ({ app }) => {
    await openFromPalette(app, 'battery load', /Battery/);
    await app.getByRole('button', { name: /^Q34 ·/ }).click();
    await expect(field(app, 'N_s')).toHaveValue('103');
    await expect(answer(app)).toHaveText(/^77\.887\d* A$/);
    await expect(app.locator('.a-note')).toContainText('4814.6');
});

test('Q378: a tool script from Topics runs with the legacy rules', async ({ app }) => {
    await app.getByRole('button', { name: 'Topics' }).click();
    await app.getByRole('textbox', { name: 'Search the library' }).fill('event score');
    await app.locator('.detail').getByRole('button', { name: /^Dynamic event score/ }).click();
    await choice(app, 'event').selectOption('skidpad');
    await field(app, 't_team').fill('5.6');
    await field(app, 't_min').fill('5.1');
    await choice(app, 'rules').selectOption('legacy');
    await app.getByRole('button', { name: /^Run/ }).click();
    await expect(answer(app)).toHaveText('41.117');
    // the answer says which rule set it comes from and what the other years would give
    await expect(app.locator('.answer .pill')).toHaveText('rules legacy');
    await expect(app.locator('.a-years')).toHaveText(/^Other rules: 2027: [\d.]+ · 2026: [\d.]+$/);
});

test('a long formula keeps the answer on screen without scrolling', async ({ app }) => {
    await app.setViewportSize({ width: 1280, height: 520 });
    await openFromPalette(app, 'engine geometry', /Displacement/);
    expect(await app.locator('label.field').count()).toBeGreaterThanOrEqual(8);
    await field(app, 'n_cyl').fill('4');
    await field(app, 'B_bore').fill('80 mm');
    await field(app, 'S_k').fill('60 mm');
    await field(app, 'omega_m').fill('6000 rpm');
    await field(app, 'bmep').fill('12 bar');
    // back at the top the last fields are below the fold, yet the answer stays fully on screen
    await app.locator('.view').first().evaluate((el) => el.scrollTo(0, 0));
    await expect(app.locator('label.field').last()).not.toBeInViewport({ ratio: 1 });
    await expect(app.locator('.answer.sticky')).toBeInViewport({ ratio: 1 });
    await expect(app.locator('.answer .pill')).toHaveCount(0);
});

test('a tool form speaks plainly: labels, a checkbox for each on/off setting, the answer follows the boxes', async ({ app }) => {
    await openFromPalette(app, 'can frame', /Bits per CAN frame/);
    await expect(app.getByText('Extended frame (29-bit ID)')).toBeVisible();
    await expect(app.locator('label.field[data-var="stuffing"] .hint')).toContainText('five equal bits');
    await app.getByRole('button', { name: /^Run/ }).click();
    // standard frame, 8 data bytes: 44 + 64 + 3 gap bits
    await expect(answer(app)).toHaveText('111');
    await switchBox(app, 'extended').check();
    await app.getByRole('button', { name: /^Run/ }).click();
    // extended: 64 + 64 + 3
    await expect(answer(app)).toHaveText('131');
    await switchBox(app, 'stuffing').check();
    await app.getByRole('button', { name: /^Run/ }).click();
    // worst-case stuff bits: floor((128 - 14) / 4) = 28 more
    await expect(answer(app)).toHaveText('159');
});

test('a choice is a list to pick from, and the rules year starts filled in from Settings', async ({ app }) => {
    await openFromPalette(app, 'skidpad score', /Dynamic event score/);
    await expect(choice(app, 'event')).toHaveValue('');
    await expect(choice(app, 'rules')).not.toHaveValue('');
    await choice(app, 'event').selectOption({ label: 'Skidpad' });
    await expect(choice(app, 'event')).toHaveValue('skidpad');
});

test('typing advice sits under its field instead of being cut off inside the label', async ({ app }) => {
    await openFromPalette(app, 'wheel loads corner', /Wheel loads in a corner from total mass/);
    const box = app.locator('label.field[data-var="ay"]');
    await expect(box.locator('.hint')).toContainText('1.5g0');
    const clipped = await box.locator('.fdesc').evaluate((el) => el.scrollWidth > el.clientWidth + 1);
    expect(clipped).toBe(false);
});

test('Q34: multiple-choice check and the copy format follow Settings', async ({ app }) => {
    await openFromPalette(app, 'battery load', /Battery/);
    await app.getByRole('button', { name: /^Q34 ·/ }).click();
    await expect(answer(app)).toHaveText(/^77\.887\d* A$/);
    await app.getByRole('button', { name: 'Check against options' }).click();
    // options in another prefix of the same unit, with a space as thousands separator
    await app.getByPlaceholder(/a\) 73\.4 A/).fill('a) 73 400 mA\nb) 77 887 mA\nc) 80.4 A');
    await expect(app.locator('.opts li.best')).toContainText('b) 77 887 mA');
    await app.getByPlaceholder(/a\) 73\.4 A/).fill('a) 73.4 A\nb) 77.9 A\nc) 80.4 A\nd) 4815 A');
    await expect(app.locator('.opts li.best')).toContainText('b) 77.9 A');

    await app.getByRole('button', { name: 'Settings' }).click();
    await app.getByRole('combobox').nth(1).selectOption('decimals');
    await app.getByRole('spinbutton').fill('1');
    await app.getByRole('checkbox', { name: /Decimal comma/ }).check();
    // back to Solve: the sheet must still hold the worked example's values and the options typed before
    await app.getByRole('button', { name: 'Solve' }).click();
    await expect(field(app, 'N_s')).toHaveValue('103');
    await expect(app.locator('.opts li.best')).toContainText('b) 77.9 A');
    await app.getByRole('button', { name: /Copy 77,9/ }).click();
    expect((await copied(app)).at(-1)).toBe('77,9');
});

test('Q515: Chain finds the rollover speed through two formulas', async ({ app }) => {
    await app.getByRole('button', { name: 'Chain' }).click();
    await app.getByRole('combobox', { name: 'Target' }).fill('v');
    await app.getByRole('textbox', { name: 'Show in' }).fill('km/h');
    await app.locator('textarea').fill('h_cg = 0.205 m\nR_c = 14.5 m\nt_tr = 1.24 m');
    await app.getByRole('button', { name: 'Chain', exact: true }).last().click();
    await expect(app.locator('.answer')).toContainText(/74\.66\d* km\/h/);
    await expect(app.locator('.steps')).toContainText('ay');
});

test('contradicting inputs are flagged instead of silently answered', async ({ app }) => {
    await openFromPalette(app, 'average speed', /Average speed/);
    await field(app, 's').fill('75 m');
    await field(app, 't').fill('3 s');
    await field(app, 'v_avg').fill('30 m/s');
    await expect(app.locator('.note-bad')).toContainText('contradict');
    await expect(app.locator('.note-bad')).toContainText('v_avg = s/t');
});

test('a wrong unit is explained, not swallowed', async ({ app }) => {
    await openFromPalette(app, 'average speed', /Average speed/);
    await field(app, 's').fill('3 kg');
    await expect(app.locator('.note-bad')).toContainText('needs a length');
});

test('pinning puts a script in the rail; opening it later takes the pasted problem values', async ({ app }) => {
    await openFromPalette(app, 'spring rate', /Helical spring/);
    await app.getByRole('button', { name: 'Pin', exact: true }).click();
    const rail = app.locator('aside.rail');
    await expect(rail.getByRole('button', { name: 'Helical spring rate' })).toBeVisible();
    await pasteAnywhere(app, SKIDPAD);
    // the problem's values are known once its matches are shown
    await expect(app.getByLabel('Scripts that fit')).toContainText('Max cornering speed with downforce');
    await rail.getByRole('button', { name: 'Max cornering speed with downforce' }).click();
    await expect(field(app, 'R_c')).toHaveValue('9.125 m');
});

test('typed values survive switching views', async ({ app }) => {
    await openFromPalette(app, 'average speed', /Average speed/);
    await field(app, 's').fill('75 m');
    await field(app, 't').fill('3.8 s');
    await expect(answer(app)).toHaveText(/^19\.7368\d* m\/s$/);
    await app.getByRole('button', { name: 'Topics' }).click();
    await app.getByRole('button', { name: 'Solve' }).click();
    await expect(field(app, 's')).toHaveValue('75 m');
    await expect(answer(app)).toHaveText(/^19\.7368\d* m\/s$/);
});

test('settings survive a reload (theme and answer format)', async ({ app }) => {
    await app.getByRole('button', { name: 'Settings' }).click();
    await app.getByRole('radio', { name: /Paper glass/ }).click();
    await expect(app.locator('html')).toHaveAttribute('data-theme', 'light');
    await app.getByRole('checkbox', { name: /Solid surfaces/ }).check();
    await app.waitForTimeout(400);
    await app.reload();
    await expect(app.locator('html')).toHaveAttribute('data-theme', 'light');
    await expect(app.locator('html')).toHaveAttribute('data-solid', '');
});

test('the searches the app suggests, and a script id as the CLI prints it, all find a script', async ({ app }) => {
    // the empty sheet and the palette placeholder tell users to type these
    for (const [query, title] of [
        ['spring rate', /spring/i],
        ['discharge', /discharge/i],
        ['skidpad score', /skidpad|score/i],
        ['battery_load', /battery/i],
    ] as const) {
        await app.keyboard.press('ControlOrMeta+k');
        await app.getByRole('combobox', { name: 'Search scripts' }).fill(query);
        await expect(app.getByRole('dialog', { name: 'Find a script' }).getByRole('option').first(), query).toContainText(title);
        await app.keyboard.press('Escape');
    }
});

test('pasting into a field is left to the field; only a paste outside fields replaces the problem', async ({ app }) => {
    await pasteAnywhere(app, SKIDPAD);
    await app.getByLabel('Scripts that fit').getByRole('button').first().click();
    const prevented = await field(app, 'mu').evaluate((el) => {
        const data = new DataTransfer();
        data.setData('text', '1.6');
        const e = new ClipboardEvent('paste', { clipboardData: data, bubbles: true, cancelable: true });
        el.dispatchEvent(e);
        return e.defaultPrevented;
    });
    expect(prevented).toBe(false);
    await expect(app.locator('#problem')).toHaveValue(SKIDPAD);
});
