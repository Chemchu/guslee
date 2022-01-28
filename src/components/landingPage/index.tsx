import { motion } from "framer-motion";
import { useState } from "react";
import Typewriter from 'typewriter-effect';
import Card from "../card";

const LandingPage = () => {
    return (
        <FondoImagen />
    );
}

const FondoVideo = () => {
    return (
        <motion.div className="items-center font-sans overflow-hidden" >
            <video autoPlay loop muted className='w-full h-full object-cover fixed -z-10'>
                <source src={'/video/tv.mp4'} type="video/mp4" />
            </video>
            <div className="text-white p-4 text-left items-end">
                <div className="flex text-3xl md:text-5xl lg:text-7xl">
                    <motion.div initial={{ opacity: 0 }} animate={{ opacity: 1, x: 0, transition: { duration: 1.5, ease: [0.87, 0, 0.13, 1], delay: 0 } }}
                        className="font-outfit mr-2 md:mr-4">
                        ¡Hola!
                    </motion.div>
                    <motion.div initial={{ opacity: 0 }} animate={{ opacity: 1, x: 0, transition: { duration: 1.5, ease: [0.87, 0, 0.13, 1], delay: 1 } }}
                        className="font-outfit-thin">
                        Me llamo Gustavo Lee
                    </motion.div>
                </div>
                <div className="text-lg md:text-3xl lg:text-5xl my-4">
                    <motion.div initial={{ opacity: 0 }} animate={{ opacity: 1, x: 0, transition: { duration: 1.5, ease: [0.87, 0, 0.13, 1], delay: 1.8 } }}
                        className="font-outfit-thin mr-2 md:mr-4">
                        Soy ingeniero de software
                    </motion.div>
                </div>

            </div>
        </motion.div >
    );
}

const FondoImagen = () => {
    const [firstWriterDone, setFirstWriter] = useState<boolean>(false);
    const [secondWriterDone, setSecondWriter] = useState<boolean>(false);
    const [thirdWriterDone, setThirdWriter] = useState<boolean>(false);

    return (
        <main className="bg-main bg-cover bg-no-repeat bg-center font-outfit-light w-screen h-screen overflow-hidden cursor-default">
            <div className="text-white m-4 text-left items-end">
                <div className="flex text-xl sm:text-4xl xl:text-6xl">
                    <div className="font-outfit mr-2 md:mr-4">
                        {!firstWriterDone ?
                            <Typewriter
                                onInit={(typewriter) => {
                                    typewriter.typeString('¡Hola!')
                                        .pauseFor(500)
                                        .stop()
                                        .callFunction(() => { setFirstWriter(true) })
                                        .start();
                                }}
                            />
                            :
                            '¡Hola!'
                        }

                    </div>
                    <div className="font-outfit-thin">
                        {firstWriterDone && !secondWriterDone ?
                            <Typewriter
                                onInit={(typewriter) => {
                                    typewriter.typeString('Soy Gus')
                                        .pauseFor(200)
                                        .deleteAll()
                                        .typeString('Me llamo Gustavo Lee')
                                        .stop()
                                        .callFunction(() => { setSecondWriter(true) })
                                        .start();
                                }}
                            />
                            :
                            secondWriterDone && 'Me llamo Gustavo Lee'
                        }
                    </div>
                </div>
                <div className="text-lg sm:text-3xl xl:text-5xl my-4">
                    <div className="flex gap-2 font-outfit-thin mr-2 md:mr-4">
                        {/* Segundo */}
                        {secondWriterDone && !thirdWriterDone ?
                            <Typewriter
                                onInit={(typewriter) => {
                                    typewriter.typeString('Soy')
                                        .pauseFor(400)
                                        .stop()
                                        .callFunction(() => { setThirdWriter(true) })
                                        .start();
                                }}
                            />
                            :
                            thirdWriterDone && 'Soy '
                        }
                        {thirdWriterDone &&
                            <Typewriter
                                options={{ deleteSpeed: 27, loop: true }}
                                onInit={(typewriter) => {
                                    typewriter
                                        .typeString('ingeniero de software')
                                        .pauseFor(1200)
                                        .deleteChars(22)
                                        .typeString('guitarrista')
                                        .pauseFor(700)
                                        .deleteChars(11)
                                        .typeString('brasileño')
                                        .pauseFor(800)
                                        .deleteChars(9)
                                        .typeString('informático')
                                        .pauseFor(900)
                                        .deleteChars(12)
                                        .typeString('fullstack developer')
                                        .pauseFor(1200)
                                        .deleteChars(19)
                                        .typeString('panadero')
                                        .pauseFor(500)
                                        .deleteChars(8)
                                        .typeString('gracioso')
                                        .pauseFor(1200)
                                        .deleteChars(8)
                                        .typeString('carismático')
                                        .pauseFor(950)
                                        .deleteChars(11)
                                        .stop()
                                        .start();
                                }}
                            />
                        }
                    </div>
                </div>
            </div>
            {/* <div className="w-full h-full p-4">
                <motion.div initial={{ opacity: 0 }} animate={{ opacity: 1, transition: { duration: 1 } }}
                    className="flex w-full h-4/5 rounded-xl bg-orange-400 shadow">
                    <Card />

                </motion.div>
            </div> */}
        </main >
    );
}

export default LandingPage;