// What a teammate does during a quiz, end to end against the real engine. Expected values are official FS-Quiz
// answers (question id in the test name) unless stated.
import { answer, copied, expect, field, openFromPalette, pasteAnywhere, test } from './fixtures';

const SKIDPAD =
    'A Formula Student car of mass 240 kg with a lift coefficient times area of 3.2 m² drives the skidpad ' +
    '(centreline radius 9.125 m). With a tyre friction coefficient of 1.4 and air density 1.1 kg/m³, what is the ' +
    'maximum speed?';

test('Q90: paste a problem anywhere, open the top match pre-filled, copy the answer', async ({ app }) => {
    await pasteAnywhere(app, SKIDPAD);
    await expect(app.getByLabel('Values found in the problem')).toContainText('3.2 m²');
    const top = app.getByLabel('Scripts that fit').getByRole('button').first();
    await expect(top).toContainText('Cornering downforce');
    await expect(top).toContainText('5');
    await top.click();
    await expect(field(app, 'mu')).toHaveValue('1.4');
    await expect(field(app, 'm')).toHaveValue('240 kg');
    await expect(answer(app)).toHaveText(/^11\.759\d* m\/s$/);
    await app.getByRole('button', { name: /Copy 11\.76/ }).click();
    expect(await copied(app)).toEqual(['11.76']);
});

test('Q245: palette search, type the knowns, pick the root that answers the question', async ({ app }) => {
    await openFromPalette(app, 'discharge', /TS discharge/);
    await field(app, 'V_0').fill('396 V');
    await field(app, 'V_c').fill('60 V');
    await field(app, 't').fill('5 s');
    await field(app, 'C').fill('1800 uF');
    const peak = app.locator('.results .res').filter({ hasText: 'P_pk' });
    await expect(peak).toContainText('106.53');
    await peak.click();
    await expect(answer(app)).toHaveText(/^106\.53\d* W$/);
    // the discharge resistor itself is also solved
    await expect(app.locator('.results .res').filter({ hasText: /^R/ })).toContainText('1472');
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
    await app.getByRole('textbox', { name: 'Filter scripts' }).fill('event score');
    await app.getByRole('button', { name: /Manual dynamic event score/ }).click();
    await field(app, 'event').fill('skidpad');
    await field(app, 't_team').fill('5.6');
    await field(app, 't_min').fill('5.1');
    await field(app, 'rules').fill('legacy');
    await app.getByRole('button', { name: /^Run/ }).click();
    await expect(answer(app)).toHaveText('41.117');
});

test('Q34: multiple-choice check and the copy format follow Settings', async ({ app }) => {
    await openFromPalette(app, 'battery load', /Battery/);
    await app.getByRole('button', { name: /^Q34 ·/ }).click();
    await expect(answer(app)).toHaveText(/^77\.887\d* A$/);
    await app.getByRole('button', { name: 'Check against options' }).click();
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
    await expect(app.locator('.note-bad')).toContainText('expected m');
});

test('pinning puts a script in the rail; opening it later takes the pasted problem values', async ({ app }) => {
    await openFromPalette(app, 'spring rate', /Helical spring/);
    await app.getByRole('button', { name: 'Pin', exact: true }).click();
    const rail = app.locator('aside.rail');
    await expect(rail.getByRole('button', { name: 'Coil spring' })).toBeVisible();
    await pasteAnywhere(app, SKIDPAD);
    // the problem's values are known once its matches are shown
    await expect(app.getByLabel('Scripts that fit')).toContainText('Cornering downforce');
    await rail.getByRole('button', { name: 'Cornering downforce' }).click();
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
